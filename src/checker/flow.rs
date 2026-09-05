use std::sync::Arc;

use crate::ast::{FlowFlags, FlowNode, Node, NodeData, Symbol, SyntaxKind};

use super::checker::Checker;
use super::types::*;

pub(crate) const FLOW_MAX_DEPTH: u32 = 2000;

#[derive(Clone)]
pub(crate) enum FlowRef {
    Symbol(Arc<Symbol>),
    Node(Arc<Node>),
}

mod narrow_expr;
mod narrow_binary;
mod narrow_discriminant;
mod narrow_calls;
mod union_ops;
impl FlowRef {

    fn anchor_node(&self) -> Option<Arc<Node>> {
        match self {
            FlowRef::Node(n) => Some(Arc::clone(n)),
            FlowRef::Symbol(s) => s.declarations.first().map(Arc::clone),
        }
    }
}

#[derive(Default)]
pub(crate) struct FlowQuery {
    memo: std::collections::HashMap<usize, Arc<Type>>,
    on_path: std::collections::HashSet<usize>,

    reduce_labels: Vec<(std::sync::Arc<FlowNode>, Vec<std::sync::Arc<FlowNode>>)>,

    loop_stack: Vec<(usize, Vec<Arc<Type>>)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NarrowKind {

    TrueBranch,

    FalseBranch,
}

impl Checker {

    pub fn get_narrowed_type_of_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        flow: Option<&Arc<FlowNode>>,
    ) -> Arc<Type> {

        let frame_type = self
            .logical_rhs_narrowing_frames
            .iter()
            .rev()
            .find(|(s, _)| Arc::ptr_eq(s, symbol))
            .map(|(_, t)| Arc::clone(t));
        let declared = self.get_type_of_symbol(symbol);
        let Some(flow) = flow else {
            return frame_type.unwrap_or(declared);
        };
        if self.flow_analysis_disabled {
            return frame_type.unwrap_or(declared);
        }
        let declared = match frame_type {
            Some(t) => t,
            None => declared,
        };
        let target = FlowRef::Symbol(Arc::clone(symbol));
        let key = self.flow_cache_key(&target, flow, &declared);
        if let Some(cached) = self.flow_type_cache.get(&key) {
            return Arc::clone(cached);
        }
        self.flow_type_cache.insert(key, Arc::clone(&declared));
        let mut query = FlowQuery::default();
        let narrowed = self.type_at_flow_node(&declared, &declared, flow, &target, 0, &mut query);
        self.flow_type_cache.insert(key, Arc::clone(&narrowed));
        narrowed
    }

    pub fn get_flow_type_of_reference(
        &mut self,
        reference: &Arc<Node>,
        declared: &Arc<Type>,
    ) -> Arc<Type> {
        let Some(flow) = self.program.symbol_map().flow_node_of(reference).map(Arc::clone)
        else {
            return Arc::clone(declared);
        };
        if self.flow_analysis_disabled {
            return Arc::clone(declared);
        }
        let target = FlowRef::Node(Arc::clone(reference));
        let key = self.flow_cache_key(&target, &flow, declared);
        if let Some(cached) = self.flow_type_cache.get(&key) {
            return Arc::clone(cached);
        }
        self.flow_type_cache.insert(key, Arc::clone(declared));
        let mut query = FlowQuery::default();
        let narrowed = self.type_at_flow_node(declared, declared, &flow, &target, 0, &mut query);
        self.flow_type_cache.insert(key, Arc::clone(&narrowed));

        if let Some(parent) = &reference.parent {
            if parent.kind == SyntaxKind::NonNullExpression
                && !narrowed.flags.contains(TypeFlags::Never)
                && self.type_is_never_after_removing_nullable(&narrowed)
            {
                return Arc::clone(declared);
            }
        }
        narrowed
    }

    fn type_is_never_after_removing_nullable(&self, t: &Arc<Type>) -> bool {
        if !self.strict_null_checks {
            return false;
        }
        if t.is_union() {
            return self.constituent_types(t).iter().all(|c| {
                c.flags.intersects(TypeFlags::Null | TypeFlags::Undefined)
            });
        }
        t.flags.intersects(TypeFlags::Null | TypeFlags::Undefined)
    }

    pub fn get_definite_assignment_flow_type(
        &mut self,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        if self.flow_analysis_disabled {
            return None;
        }
        let flow = self
            .program
            .symbol_map()
            .flow_node_of(node)
            .map(Arc::clone)?;
        let declared = self.get_type_of_symbol(symbol);
        let undefined = self.undefined_type();
        let initial = if self.type_contains_undefined_local(&declared) {
            Arc::clone(&declared)
        } else {
            self.get_union_type(vec![Arc::clone(&declared), undefined])
        };
        let target = FlowRef::Symbol(Arc::clone(symbol));
        let key = self.flow_cache_key(&target, &flow, &initial);
        if let Some(cached) = self.flow_type_cache.get(&key) {
            return Some(Arc::clone(cached));
        }
        self.flow_type_cache.insert(key, Arc::clone(&declared));
        let mut query = FlowQuery::default();
        let narrowed = self.type_at_flow_node(&declared, &initial, &flow, &target, 0, &mut query);
        self.flow_type_cache.insert(key, Arc::clone(&narrowed));
        Some(narrowed)
    }

    fn type_contains_undefined_local(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Undefined) {
            return true;
        }
        if t.is_union() {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|c| c.flags.contains(TypeFlags::Undefined));
            }
        }
        false
    }

    fn flow_cache_key(&self, target: &FlowRef, flow: &Arc<FlowNode>, initial: &Arc<Type>) -> u64 {
        let ref_part = match target {
            FlowRef::Symbol(symbol) => symbol.id(),
            FlowRef::Node(node) => node.id(),
        };
        let flow_ptr = Arc::as_ptr(flow) as *const FlowNode as u64;
        let initial_ptr = Arc::as_ptr(initial) as *const Type as u64;

        (ref_part.rotate_left(17) ^ flow_ptr).rotate_left(29) ^ initial_ptr
    }

    fn type_at_flow_node(
        &mut self,
        declared: &Arc<Type>,
        initial: &Arc<Type>,
        flow: &Arc<FlowNode>,
        target: &FlowRef,
        depth: u32,
        query: &mut FlowQuery,
    ) -> Arc<Type> {
        let key = Arc::as_ptr(flow) as usize;
        if let Some(t) = query.memo.get(&key) {
            return Arc::clone(t);
        }
        if !query.on_path.insert(key) {

            for (_, types) in query.loop_stack.iter().rev() {
                if !types.is_empty() {
                    if types.len() == 1 {
                        return Arc::clone(&types[0]);
                    }
                    return self.get_union_type(types.clone());
                }
            }
            query.memo.insert(key, Arc::clone(initial));
            return Arc::clone(initial);
        }
        let result = if depth >= FLOW_MAX_DEPTH {

            if !self.flow_analysis_disabled {
                self.flow_analysis_disabled = true;
                self.report_flow_control_error(target);
            }

            self.error_type()
        } else {
            self.compute_type_at_flow_node(declared, initial, flow, target, depth, query)
        };
        query.on_path.remove(&key);
        query.memo.insert(key, Arc::clone(&result));
        result
    }

    fn report_flow_control_error(&mut self, target: &FlowRef) {
        use crate::ast::SyntaxKind;
        let Some(anchor) = target.anchor_node() else { return };
        let mut block: Option<Arc<Node>> = None;
        let mut cur = anchor.parent.clone();
        while let Some(n) = cur {
            let is_function_or_module_block = match n.kind {
                SyntaxKind::SourceFile | SyntaxKind::ModuleBlock => true,
                SyntaxKind::Block => n
                    .parent
                    .as_ref()
                    .is_some_and(|p| crate::ast::utilities::is_function_like_kind(p.kind)),
                _ => false,
            };
            if is_function_or_module_block {
                block = Some(Arc::clone(&n));
                break;
            }
            cur = n.parent.clone();
        }
        let Some(block) = block else { return };

        let mut loc = block.loc;
        if let Some(stmts) = match &block.data {
            crate::ast::NodeData::SourceFile(d) => Some(&d.statements),
            crate::ast::NodeData::ModuleBlock(d) => Some(&d.statements),
            crate::ast::NodeData::Block(d) => Some(&d.statements),
            _ => None,
        } && let Some(first) = stmts.nodes.first()
        {
            loc = crate::core::text::TextRange::new(first.loc.pos as usize, first.loc.pos as usize + 1);
        }
        self.diagnostics.add(crate::ast::Diagnostic::new(
            self.current_file.clone(),
            loc,
            crate::diagnostics::messages_generated::
                THE_CONTAINING_FUNCTION_OR_MODULE_BODY_IS_TOO_LARGE_FOR_CONTROL_FLOW_ANALYSIS,
            Vec::new(),
        ));
    }

    fn compute_type_at_flow_node(
        &mut self,
        declared: &Arc<Type>,
        initial: &Arc<Type>,
        flow: &Arc<FlowNode>,
        target: &FlowRef,
        depth: u32,
        query: &mut FlowQuery,
    ) -> Arc<Type> {

        if flow.flags.contains(FlowFlags::UNREACHABLE) {
            return self.never_type();
        }

        if flow.flags.contains(FlowFlags::START) {
            return Arc::clone(initial);
        }

        if flow.flags.intersects(FlowFlags::CONDITION) {
            let kind = if flow.flags.contains(FlowFlags::TRUE_CONDITION) {
                NarrowKind::TrueBranch
            } else {
                NarrowKind::FalseBranch
            };
            let antecedent_type = self.antecedent_type_at(declared, initial, flow, target, depth, query);
            if let Some(expr) = &flow.node {
                return self.narrow_by_expression(&antecedent_type, expr, target, kind, depth);
            }
            return antecedent_type;
        }

        if flow.flags.contains(FlowFlags::ASSIGNMENT) {
            if let Some(expr) = &flow.node {
                if let Some(t) = self.assignment_flow_type(expr, target, declared) {
                    return t;
                }

                if let FlowRef::Node(reference) = target {
                    if self.contains_matching_reference(reference, expr) {
                        return Arc::clone(declared);
                    }
                }

                if expr.kind == SyntaxKind::VariableDeclaration {
                    if let Some(for_in_expr) = Self::for_in_expression_of(expr) {
                        if self.expr_matches_target(&for_in_expr, target)
                            || self.optional_chain_contains_target(&for_in_expr, target)
                        {
                            let ante = self.antecedent_type_at(
                                declared, initial, flow, target, depth, query,
                            );
                            if self.strict_null_checks {
                                return self.remove_nullable_from_union(&ante);
                            }
                            return ante;
                        }
                    }
                }
            }

            return self.antecedent_type_at(declared, initial, flow, target, depth, query);
        }

        if flow.flags.contains(FlowFlags::SWITCH_CLAUSE) {
            let antecedent_type =
                self.antecedent_type_at(declared, initial, flow, target, depth, query);
            return self.narrow_by_switch_clause(&antecedent_type, flow, target);
        }

        if flow.flags.contains(FlowFlags::ARRAY_MUTATION) {
            if let Some(node) = &flow.node {
                let is_evolving = declared.object_flags.contains(ObjectFlags::EvolvingArray)
                    || self.is_auto_array_type(declared);
                if is_evolving {
                    let pre_type = self.antecedent_type_at(declared, initial, flow, target, depth, query);
                    if let Some(evolved) =
                        self.evolve_array_at_mutation(node, &pre_type, target)
                    {
                        return evolved;
                    }
                    return pre_type;
                }
            }

            return self.antecedent_type_at(declared, initial, flow, target, depth, query);
        }

        if flow.flags.contains(FlowFlags::CALL) {
            let antecedent_type = self.antecedent_type_at(declared, initial, flow, target, depth, query);
            if let Some(call_expr) = &flow.node {
                return self.narrow_by_assertion_call(&antecedent_type, call_expr, target);
            }
            return antecedent_type;
        }

        if flow.flags.contains(FlowFlags::REDUCE_LABEL) {
            if let Some(reduce_target) = &flow.reduce_target {
                query.reduce_labels.push((
                    Arc::clone(reduce_target),
                    flow.antecedents.clone(),
                ));
                let t = self.antecedent_type_at(declared, initial, flow, target, depth, query);
                query.reduce_labels.pop();
                return t;
            }
            return self.antecedent_type_at(declared, initial, flow, target, depth, query);
        }

        if flow.flags.contains(FlowFlags::LOOP_LABEL) && flow.antecedents.len() > 1 {
            let key = Arc::as_ptr(flow) as usize;
            if let Some((_, types)) = query.loop_stack.iter().rev().find(|(k, _)| *k == key) {
                if !types.is_empty() {
                    let distinct: Vec<Arc<Type>> = types.clone();
                    if distinct.len() == 1 {
                        return distinct.into_iter().next().expect("exactly one");
                    }
                    return self.get_union_type(distinct);
                }
            }
            let mut ant_types: Vec<Arc<Type>> = Vec::new();
            for ant in &flow.antecedents {
                query.loop_stack.push((key, ant_types.clone()));
                let t =
                    self.type_at_flow_node(declared, initial, ant, target, depth + 1, query);
                query.loop_stack.pop();
                if !ant_types.iter().any(|u| Arc::ptr_eq(u, &t)) {
                    ant_types.push(t.clone());
                }
                if Arc::ptr_eq(&t, declared) {
                    break;
                }
            }
            if ant_types.len() == 1 {
                return ant_types.into_iter().next().expect("exactly one");
            }
            if ant_types.is_empty() {
                return Arc::clone(initial);
            }
            return self.get_union_type(ant_types);
        }

        if flow.antecedents.len() > 1 {
            let effective: Vec<Arc<FlowNode>> = query
                .reduce_labels
                .iter()
                .rev()
                .find(|(reduce_target, _)| Arc::ptr_eq(reduce_target, flow))
                .map(|(_, ants)| ants.clone())
                .unwrap_or_else(|| flow.antecedents.clone());
            if effective.len() == 1 {
                return self.type_at_flow_node(
                    declared,
                    initial,
                    &effective[0],
                    target,
                    depth + 1,
                    query,
                );
            }
            let mut antecedent_types: Vec<Arc<Type>> = Vec::new();
            for antecedent in &effective {
                let t =
                    self.type_at_flow_node(declared, initial, antecedent, target, depth + 1, query);
                if !antecedent_types.iter().any(|u| Arc::ptr_eq(u, &t)) {
                    antecedent_types.push(t);
                }
            }

            if antecedent_types.len() == 1 {
                return antecedent_types.into_iter().next().expect("exactly one");
            }

            if antecedent_types.is_empty() {
                return Arc::clone(declared);
            }
            return self.get_union_type(antecedent_types);
        }

        self.antecedent_type_at(declared, initial, flow, target, depth, query)
    }

    fn antecedent_type_at(
        &mut self,
        declared: &Arc<Type>,
        initial: &Arc<Type>,
        flow: &Arc<FlowNode>,
        target: &FlowRef,
        depth: u32,
        query: &mut FlowQuery,
    ) -> Arc<Type> {
        let antecedent = flow
            .antecedent
            .as_ref()
            .or_else(|| flow.antecedents.first());
        match antecedent {
            Some(antecedent) => {
                self.type_at_flow_node(declared, initial, antecedent, target, depth + 1, query)
            }
            None => Arc::clone(initial),
        }
    }





    pub(super) fn constituent_types(&self, type_: &Arc<Type>) -> Vec<Arc<Type>> {
        if type_.is_union() {
            if let TypeData::Union(u) = &type_.data {
                return u.union_or_intersection.types.clone();
            }
        }
        if type_.flags.contains(TypeFlags::Never) {
            return Vec::new();
        }
        vec![Arc::clone(type_)]
    }

}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PropertyPresence {

    Definitely,

    Maybe,

    DefinitelyNot,
}

impl PropertyPresence {
    fn is_definitely(self) -> bool {
        matches!(self, PropertyPresence::Definitely)
    }
    fn is_definitely_not(self) -> bool {
        matches!(self, PropertyPresence::DefinitelyNot)
    }
}

pub(crate) fn is_assignment_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::EqualsToken
            | SyntaxKind::PlusEqualsToken
            | SyntaxKind::MinusEqualsToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
            | SyntaxKind::AmpersandEqualsToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::CaretEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken
    )
}

pub(crate) fn clauses_of_range(switch_stmt: &Arc<Node>, start: usize, end: usize) -> Vec<Arc<Node>> {
    let NodeData::SwitchStatement(sd) = &switch_stmt.data else {
        return Vec::new();
    };
    let NodeData::CaseBlock(cb) = &sd.case_block.data else {
        return Vec::new();
    };
    let clauses = &cb.clauses.nodes;
    let start = start.min(clauses.len());
    let end = end.max(start).min(clauses.len());
    clauses[start..end].to_vec()
}
