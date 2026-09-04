use std::sync::Arc;

use crate::ast::{FlowFlags, FlowNode, Node, NodeData, NodeFlags, Symbol, SymbolFlags, SyntaxKind};

use super::checker::Checker;
use super::types::*;

const FLOW_MAX_DEPTH: u32 = 2000;

#[derive(Clone)]
pub(super) enum FlowRef {
    Symbol(Arc<Symbol>),
    Node(Arc<Node>),
}

impl FlowRef {

    fn anchor_node(&self) -> Option<Arc<Node>> {
        match self {
            FlowRef::Node(n) => Some(Arc::clone(n)),
            FlowRef::Symbol(s) => s.declarations.first().map(Arc::clone),
        }
    }
}

#[derive(Default)]
struct FlowQuery {
    memo: std::collections::HashMap<usize, Arc<Type>>,
    on_path: std::collections::HashSet<usize>,

    reduce_labels: Vec<(std::sync::Arc<FlowNode>, Vec<std::sync::Arc<FlowNode>>)>,

    loop_stack: Vec<(usize, Vec<Arc<Type>>)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum NarrowKind {

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

    fn narrow_by_expression(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
        depth: u32,
    ) -> Arc<Type> {

        if expr.kind == SyntaxKind::ParenthesizedExpression {
            if let NodeData::ParenthesizedExpression(p) = &expr.data {
                return self.narrow_by_expression(type_, &p.expression, target, kind, depth);
            }
        }

        if expr.kind == SyntaxKind::BinaryExpression {
            if let NodeData::BinaryExpression(bin) = &expr.data {
                if bin.operator_token.kind == SyntaxKind::AmpersandAmpersandToken {
                    if kind == NarrowKind::TrueBranch {
                        let narrowed =
                            self.narrow_by_expression(type_, &bin.left, target, kind, depth);
                        return self
                            .narrow_by_expression(&narrowed, &bin.right, target, kind, depth);
                    }

                    if kind == NarrowKind::FalseBranch {
                        let a_false = self.narrow_by_expression(
                            type_,
                            &bin.left,
                            target,
                            NarrowKind::FalseBranch,
                            depth,
                        );
                        let a_true = self.narrow_by_expression(
                            type_,
                            &bin.left,
                            target,
                            NarrowKind::TrueBranch,
                            depth,
                        );
                        let b_false = self.narrow_by_expression(
                            &a_true,
                            &bin.right,
                            target,
                            NarrowKind::FalseBranch,
                            depth,
                        );
                        return self.flow_union_of(&[a_false, b_false]);
                    }

                    let narrowed =
                        self.narrow_by_expression(type_, &bin.left, target, kind, depth);
                    return self
                        .narrow_by_expression(&narrowed, &bin.right, target, kind, depth);
                }
                if bin.operator_token.kind == SyntaxKind::BarBarToken {
                    if kind == NarrowKind::FalseBranch {

                        let narrowed =
                            self.narrow_by_expression(type_, &bin.left, target, kind, depth);
                        return self
                            .narrow_by_expression(&narrowed, &bin.right, target, kind, depth);
                    }

                    let a_true = self.narrow_by_expression(
                        type_,
                        &bin.left,
                        target,
                        NarrowKind::TrueBranch,
                        depth,
                    );
                    let a_false = self.narrow_by_expression(
                        type_,
                        &bin.left,
                        target,
                        NarrowKind::FalseBranch,
                        depth,
                    );
                    let b_true = self.narrow_by_expression(
                        &a_false,
                        &bin.right,
                        target,
                        NarrowKind::TrueBranch,
                        depth,
                    );
                    return self.flow_union_of(&[a_true, b_true]);
                }
                if bin.operator_token.kind == SyntaxKind::QuestionQuestionToken {

                    if kind == NarrowKind::TrueBranch {

                        return Arc::clone(type_);
                    }

                    let narrowed =
                        self.narrow_by_optionality(type_, &bin.left, target, kind, depth);
                    return self.narrow_by_expression(&narrowed, &bin.right, target, kind, depth);
                }
            }
        }

        if expr.kind == SyntaxKind::PrefixUnaryExpression {
            if let NodeData::PrefixUnaryExpression(unary) = &expr.data {
                if unary.operator == SyntaxKind::ExclamationToken {
                    let inverted = if kind == NarrowKind::TrueBranch {
                        NarrowKind::FalseBranch
                    } else {
                        NarrowKind::TrueBranch
                    };
                    return self.narrow_by_expression(
                        type_,
                        &unary.operand,
                        target,
                        inverted,
                        depth,
                    );
                }
            }
        }

        if expr.kind == SyntaxKind::BinaryExpression {
            return self.narrow_by_binary(type_, expr, target, kind);
        }

        if expr.kind == SyntaxKind::CallExpression {
            return self.narrow_by_call_expression(type_, expr, target, kind);
        }

        if expr.kind == SyntaxKind::Identifier
            && !self.expr_matches_target(expr, target)
            && self.flow_inline_level < 5
        {
            if let Some(init_expr) = self.const_alias_initializer(expr) {
                self.flow_inline_level += 1;
                let result = self.narrow_by_expression(type_, &init_expr, target, kind, depth);
                self.flow_inline_level -= 1;
                return result;
            }
        }

        if self.expr_matches_target(expr, target) {
            return self.narrow_by_truthiness(type_, kind);
        }

        if kind == NarrowKind::TrueBranch {
            let contains = self.optional_chain_contains_target(expr, target);
            if contains {
                return self.remove_nullable_from_union(type_);
            }
        }

        if let Some(name) = self.discriminant_property_name_on_target(expr, target) {
            return self.narrow_by_property_truthiness(type_, &name, kind);
        }

        Arc::clone(type_)
    }

    fn discriminant_property_name_on_target(
        &self,
        expr: &Arc<Node>,
        target: &FlowRef,
    ) -> Option<String> {
        match &expr.data {
            NodeData::PropertyAccessExpression(pa) => {
                if self.expr_matches_target(&pa.expression, target) {
                    Some(pa.name.text().to_string())
                } else {
                    None
                }
            }
            NodeData::ElementAccessExpression(ea) => {
                if self.expr_matches_target(&ea.expression, target) {
                    match &ea.argument_expression.data {
                        NodeData::StringLiteral(s) => Some(s.text.clone()),
                        NodeData::NumericLiteral(n) => Some(n.text.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn narrow_by_property_truthiness(
        &mut self,
        type_: &Arc<Type>,
        name: &str,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let constituents = match type_.flags.contains(TypeFlags::Union) {
            true => match &type_.data {
                TypeData::Union(u) => u.union_or_intersection.types.clone(),
                _ => return Arc::clone(type_),
            },
            false => return Arc::clone(type_),
        };
        let mut kept: Vec<Arc<Type>> = Vec::with_capacity(constituents.len());
        for c in &constituents {
            let prop_type = match self.get_constituent_property(c, name) {
                Some(sym) => self.get_type_of_symbol(&sym),
                None => {
                    kept.push(Arc::clone(c));
                    continue;
                }
            };
            let undecidable = prop_type.flags.intersects(
                TypeFlags::Any
                    | TypeFlags::Unknown
                    | TypeFlags::TypeParameter
                    | TypeFlags::Conditional
                    | TypeFlags::IndexedAccess,
            );
            if undecidable {
                kept.push(Arc::clone(c));
                continue;
            }
            let parts: Vec<Arc<Type>> = if prop_type.flags.contains(TypeFlags::Union) {
                prop_type.types().unwrap_or(&[]).to_vec()
            } else {
                vec![Arc::clone(&prop_type)]
            };
            let any_falsy = parts.iter().any(|p| self.constituent_is_definitely_falsy(p));
            let all_falsy = parts.iter().all(|p| self.constituent_is_definitely_falsy(p));
            match kind {

                NarrowKind::FalseBranch if any_falsy => kept.push(Arc::clone(c)),

                NarrowKind::TrueBranch if !all_falsy => kept.push(Arc::clone(c)),
                _ => {}
            }
        }
        if kept.is_empty() || kept.len() == constituents.len() {
            return Arc::clone(type_);
        }
        self.flow_union_of(&kept)
    }

    fn narrow_by_binary(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let NodeData::BinaryExpression(bin) = &expr.data else {
            return Arc::clone(type_);
        };
        let op = bin.operator_token.kind;

        if op == SyntaxKind::InstanceOfKeyword {
            return self.narrow_by_instanceof(type_, &bin.left, &bin.right, target, kind);
        }

        if op == SyntaxKind::InKeyword {
            return self.narrow_by_in_keyword(type_, &bin.left, &bin.right, target, kind);
        }

        if matches!(
            op,
            SyntaxKind::AmpersandAmpersandToken | SyntaxKind::BarBarToken
        ) {
            let is_and = op == SyntaxKind::AmpersandAmpersandToken;
            return match kind {
                NarrowKind::TrueBranch if is_and => {
                    let t =
                        self.narrow_by_binary(type_, &bin.left, target, NarrowKind::TrueBranch);
                    self.narrow_by_binary(&t, &bin.right, target, NarrowKind::TrueBranch)
                }
                NarrowKind::FalseBranch if is_and => {
                    let a =
                        self.narrow_by_binary(type_, &bin.left, target, NarrowKind::FalseBranch);
                    let b =
                        self.narrow_by_binary(type_, &bin.right, target, NarrowKind::FalseBranch);
                    self.flow_union_of(&[a, b])
                }
                NarrowKind::TrueBranch => {
                    let a =
                        self.narrow_by_binary(type_, &bin.left, target, NarrowKind::TrueBranch);
                    let b =
                        self.narrow_by_binary(type_, &bin.right, target, NarrowKind::TrueBranch);
                    self.flow_union_of(&[a, b])
                }
                NarrowKind::FalseBranch => {
                    let t =
                        self.narrow_by_binary(type_, &bin.left, target, NarrowKind::FalseBranch);
                    self.narrow_by_binary(&t, &bin.right, target, NarrowKind::FalseBranch)
                }
            };
        }

        let is_strict = op == SyntaxKind::EqualsEqualsEqualsToken
            || op == SyntaxKind::ExclamationEqualsEqualsToken;
        let is_loose =
            op == SyntaxKind::EqualsEqualsToken || op == SyntaxKind::ExclamationEqualsToken;
        if !is_strict && !is_loose {
            return Arc::clone(type_);
        }

        let is_equality =
            op == SyntaxKind::EqualsEqualsEqualsToken || op == SyntaxKind::EqualsEqualsToken;

        let narrow_to_value = if is_equality {
            kind == NarrowKind::TrueBranch
        } else {
            kind == NarrowKind::FalseBranch
        };

        if bin.left.kind == SyntaxKind::TypeOfExpression
            && self.typeof_expr_matches_target(&bin.left, target)
        {
            return self.narrow_by_typeof(type_, &bin.right, narrow_to_value, is_loose);
        }
        if bin.right.kind == SyntaxKind::TypeOfExpression
            && self.typeof_expr_matches_target(&bin.right, target)
        {
            return self.narrow_by_typeof(type_, &bin.left, narrow_to_value, is_loose);
        }

        if bin.left.kind == SyntaxKind::TypeOfExpression {
            if let Some(narrowed) = self.try_narrow_by_typeof_discriminant(
                type_,
                &bin.left,
                &bin.right,
                target,
                narrow_to_value,
            ) {
                return narrowed;
            }
        }
        if bin.right.kind == SyntaxKind::TypeOfExpression {
            if let Some(narrowed) = self.try_narrow_by_typeof_discriminant(
                type_,
                &bin.right,
                &bin.left,
                target,
                narrow_to_value,
            ) {
                return narrowed;
            }
        }

        if let Some(narrowed) = self.try_narrow_by_discriminant_property(type_, expr, target, kind)
        {
            return narrowed;
        }

        if self.optional_chain_contains_target(&bin.left, target) {
            return self.narrow_by_optional_chain_containment(type_, op, &bin.right, kind);
        }
        if self.optional_chain_contains_target(&bin.right, target) {
            return self.narrow_by_optional_chain_containment(type_, op, &bin.left, kind);
        }

        let (value_node, is_symbol_on_left) = if self.expr_matches_target(&bin.left, target) {
            (&bin.right, true)
        } else if self.expr_matches_target(&bin.right, target) {
            (&bin.left, false)
        } else {
            return Arc::clone(type_);
        };
        let _ = is_symbol_on_left;

        let value_type = self.get_type_of_node(value_node);
        self.narrow_by_equality(type_, &value_type, narrow_to_value, is_loose)
    }

    fn narrow_by_equality(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
        narrow_to_value: bool,
        is_loose: bool,
    ) -> Arc<Type> {

        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(type_);
        }

        if value_type.flags.contains(TypeFlags::BooleanLiteral)
            && type_.flags.contains(TypeFlags::Boolean)
            && !is_loose
        {
            let is_true_value = match value_type.literal_value() {
                Some(LiteralValue::Boolean(b)) => *b,
                _ => true,
            };
            let target_is_true = if narrow_to_value {
                is_true_value
            } else {
                !is_true_value
            };
            return if target_is_true {
                self.true_type()
            } else {
                self.false_type()
            };
        }

        if value_type.flags.intersects(TYPE_FLAGS_NULLABLE) {
            if !self.strict_null_checks {
                return Arc::clone(type_);
            }
            let value_is_null = value_type.flags.contains(TypeFlags::Null);

            return if is_loose {
                if narrow_to_value {
                    self.filter_type_by_flags(type_, TYPE_FLAGS_NULLABLE)
                } else {
                    self.remove_flags_from_union(type_, TYPE_FLAGS_NULLABLE)
                }
            } else if value_is_null {
                if narrow_to_value {
                    self.filter_type_by_flags(type_, TypeFlags::Null)
                } else {
                    self.remove_flags_from_union(type_, TypeFlags::Null)
                }
            } else {
                if narrow_to_value {
                    self.filter_type_by_flags(type_, TypeFlags::Undefined)
                } else {
                    self.remove_flags_from_union(type_, TypeFlags::Undefined)
                }
            };
        }
        if narrow_to_value {

            let filtered = self.filter_comparable_or_coercible(type_, value_type, is_loose);
            self.replace_primitives_with_literals(&filtered, value_type)
        } else {

            if !value_type.flags.intersects(TYPE_FLAGS_UNIT) {
                return Arc::clone(type_);
            }
            self.remove_comparable_units(type_, value_type)
        }
    }

    fn filter_comparable_or_coercible(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
        is_loose: bool,
    ) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let value_constituents = self.constituent_types(value_type);
        let matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {

                let comparable = value_constituents
                    .iter()
                    .any(|vc| self.is_type_comparable_to(t, vc));
                if comparable {
                    return true;
                }

                is_loose
                    && value_constituents
                        .iter()
                        .any(|vc| Self::is_coercible_under_double_equals(t, vc))
            })
            .collect();
        self.rebuild_union_or_never(type_, matching)
    }

    fn remove_comparable_units(&mut self, type_: &Arc<Type>, value_type: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let value_constituents = self.constituent_types(value_type);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {

                if !t.flags.intersects(TYPE_FLAGS_UNIT) {
                    return true;
                }

                !value_constituents
                    .iter()
                    .any(|vc| self.is_type_comparable_to(t, vc))
            })
            .collect();
        self.rebuild_union_or_never(type_, remaining)
    }

    fn replace_primitives_with_literals(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {

        let has_primitives = type_
            .flags
            .intersects(TypeFlags::String | TypeFlags::Number | TypeFlags::BigInt);
        let has_literals = value_type
            .flags
            .intersects(TYPE_FLAGS_LITERAL | TypeFlags::TemplateLiteral | TypeFlags::StringMapping);
        if !has_primitives || !has_literals {
            return Arc::clone(type_);
        }

        let value_constituents = self.constituent_types(value_type);
        let string_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| {
                t.flags.intersects(
                    TypeFlags::StringLiteral
                        | TypeFlags::TemplateLiteral
                        | TypeFlags::StringMapping,
                )
            })
            .cloned()
            .collect();
        let number_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| t.flags.contains(TypeFlags::NumberLiteral))
            .cloned()
            .collect();
        let bigint_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| t.flags.contains(TypeFlags::BigIntLiteral))
            .cloned()
            .collect();
        let constituents = self.constituent_types(type_);
        let mut result: Vec<Arc<Type>> = Vec::new();
        for t in constituents {
            if t.flags.contains(TypeFlags::String) {

                let has_string_value =
                    value_type.flags.contains(TypeFlags::String) || string_literals.is_empty();
                if has_string_value {
                    result.push(t);
                } else {
                    result.extend(string_literals.iter().cloned());
                }
            } else if t.flags.contains(TypeFlags::Number) {
                let has_number_value =
                    value_type.flags.contains(TypeFlags::Number) || number_literals.is_empty();
                if has_number_value {
                    result.push(t);
                } else {
                    result.extend(number_literals.iter().cloned());
                }
            } else if t.flags.contains(TypeFlags::BigInt) {
                let has_bigint_value =
                    value_type.flags.contains(TypeFlags::BigInt) || bigint_literals.is_empty();
                if has_bigint_value {
                    result.push(t);
                } else {
                    result.extend(bigint_literals.iter().cloned());
                }
            } else {
                result.push(t);
            }
        }
        self.rebuild_union_or_never(type_, result)
    }

    fn is_coercible_under_double_equals(source: &Arc<Type>, target: &Arc<Type>) -> bool {
        source
            .flags
            .intersects(TypeFlags::Number | TypeFlags::String | TypeFlags::BooleanLiteral)
            && target
                .flags
                .intersects(TypeFlags::Number | TypeFlags::String | TypeFlags::Boolean)
    }

    fn narrow_by_instanceof(
        &mut self,
        type_: &Arc<Type>,
        left: &Arc<Node>,
        right: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {
        if !self.expr_matches_target(left, target) {
            return Arc::clone(type_);
        }
        let right_type = self.get_type_of_node(right);
        let Some(instance_type) = self.get_instance_type_of_constructor(&right_type) else {
            return Arc::clone(type_);
        };
        match kind {
            NarrowKind::TrueBranch => self.narrow_to_subtype(type_, &instance_type),
            NarrowKind::FalseBranch => self.remove_subtype_from_union(type_, &instance_type),
        }
    }

    fn narrow_by_in_keyword(
        &mut self,
        type_: &Arc<Type>,
        left: &Arc<Node>,
        right: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {

        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(type_);
        }
        if !self.expr_matches_target(right, target) {
            return Arc::clone(type_);
        }
        let Some(prop_name) = Self::get_accessed_property_name_from_node(left) else {
            return Arc::clone(type_);
        };
        let keep_present = match kind {
            NarrowKind::TrueBranch => true,
            NarrowKind::FalseBranch => false,
        };

        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let has_prop = self.type_has_property(t, &prop_name);
                if keep_present {
                    !has_prop.is_definitely_not()
                } else {
                    !has_prop.is_definitely()
                }
            })
            .collect();
        self.rebuild_union_or_never(type_, filtered)
    }

    fn try_narrow_by_discriminant_property(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Option<Arc<Type>> {

        let (symbol, node_reference): (Option<Arc<Symbol>>, Option<Arc<Node>>) = match target {
            FlowRef::Symbol(symbol) => (Some(Arc::clone(symbol)), None),
            FlowRef::Node(reference) => (None, Some(Arc::clone(reference))),
        };
        let NodeData::BinaryExpression(bin) = &expr.data else {
            return None;
        };
        let op = bin.operator_token.kind;

        let is_strict_eq = op == SyntaxKind::EqualsEqualsEqualsToken
            || op == SyntaxKind::ExclamationEqualsEqualsToken;
        if !is_strict_eq {
            return None;
        }

        let (access_node, value_node) = if let Some(symbol) = &symbol {
            if let Some(alias) = self.discriminant_alias_access(&bin.left, symbol) {
                (alias, &bin.right)
            } else if let Some(alias) = self.discriminant_alias_access(&bin.right, symbol) {
                (alias, &bin.left)
            } else if self.is_property_access_on_symbol(&bin.left, symbol) {
                (Arc::clone(&bin.left), &bin.right)
            } else if self.is_property_access_on_symbol(&bin.right, symbol) {
                (Arc::clone(&bin.right), &bin.left)
            } else {
                return None;
            }
        } else if let Some(reference) = node_reference.as_ref() {
            if self.is_property_access_on_reference(&bin.left, reference) {
                (Arc::clone(&bin.left), &bin.right)
            } else if self.is_property_access_on_reference(&bin.right, reference) {
                (Arc::clone(&bin.right), &bin.left)
            } else {
                return None;
            }
        } else {
            unreachable!()
        };
        let prop_name = Self::get_accessed_property_name_from_node(&access_node)?;
        let value_type = self.get_type_of_node(value_node);
        let is_equality = op == SyntaxKind::EqualsEqualsEqualsToken;
        let keep_matching = if is_equality {
            kind == NarrowKind::TrueBranch
        } else {
            kind == NarrowKind::FalseBranch
        };

        if !type_.is_union() {
            let Some(prop_type) = self.get_property_type_of_type(type_, &prop_name) else {
                return Some(Arc::clone(type_));
            };
            if prop_type.flags.contains(TypeFlags::Any) {
                return Some(Arc::clone(type_));
            };

            let could_equal = self.is_type_assignable_to(&prop_type, &value_type)
                || self.is_type_assignable_to(&value_type, &prop_type);
            if keep_matching {
                return Some(if could_equal {
                    Arc::clone(type_)
                } else {
                    self.never_type()
                });
            }

            if value_type.flags.intersects(TYPE_FLAGS_UNIT)
                && self.is_type_assignable_to(&prop_type, &value_type)
            {
                return Some(self.never_type());
            }
            return Some(Arc::clone(type_));
        }
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let prop_type = self.get_property_type_of_type(t, &prop_name);

                if prop_type
                    .as_ref()
                    .is_some_and(|pt| pt.flags.contains(TypeFlags::Never))
                {
                    return false;
                }
                if keep_matching {

                    prop_type
                        .map(|pt| {
                            self.is_type_assignable_to(&pt, &value_type)
                                || self.is_type_assignable_to(&value_type, &pt)
                        })
                        .unwrap_or(false)
                } else {

                    prop_type
                        .map(|pt| !self.is_type_assignable_to(&pt, &value_type))
                        .unwrap_or(true)
                }
            })
            .collect();
        Some(self.rebuild_union_or_never(type_, filtered))
    }

    fn try_narrow_by_typeof_discriminant(
        &mut self,
        type_: &Arc<Type>,
        typeof_expr: &Arc<Node>,
        type_name_node: &Arc<Node>,
        target: &FlowRef,
        narrow_to_value: bool,
    ) -> Option<Arc<Type>> {
        let FlowRef::Symbol(symbol) = target else {
            return None;
        };
        let NodeData::TypeOfExpression(typeof_data) = &typeof_expr.data else {
            return None;
        };
        let target = &typeof_data.expression;

        let owned = match self.discriminant_alias_access(target, symbol) {
            Some(alias) => alias,
            None if self.is_property_access_on_symbol(target, symbol) => Arc::clone(target),
            None => return None,
        };
        let prop_name = Self::get_accessed_property_name_from_node(&owned)?;

        if !type_.is_union() {
            return Some(Arc::clone(type_));
        }
        let type_name = match &type_name_node.data {
            NodeData::StringLiteral(data) => data.text.as_str(),
            _ => return None,
        };
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let prop_type = self.get_property_type_of_type(t, &prop_name);
                let Some(prop_type) = prop_type else {
                    return false;
                };
                if narrow_to_value {

                    self.type_matches_typeof_any(&prop_type, type_name)
                } else {

                    !self.type_matches_typeof_all(&prop_type, type_name)
                }
            })
            .collect();
        Some(self.rebuild_union_or_never(type_, filtered))
    }

    fn type_matches_typeof_any(&self, t: &Arc<Type>, type_name: &str) -> bool {
        let constituents = self.constituent_types(t);
        constituents
            .iter()
            .any(|c| self.constituent_matches_typeof(c, type_name))
    }

    fn type_matches_typeof_all(&self, t: &Arc<Type>, type_name: &str) -> bool {
        let constituents = self.constituent_types(t);
        !constituents.is_empty()
            && constituents
                .iter()
                .all(|c| self.constituent_matches_typeof(c, type_name))
    }

    fn constituent_matches_typeof(&self, t: &Arc<Type>, type_name: &str) -> bool {
        match type_name {
            "string" => t.flags.intersects(TYPE_FLAGS_STRING_LIKE),
            "number" => t.flags.intersects(TYPE_FLAGS_NUMBER_LIKE),
            "boolean" => t.flags.intersects(TYPE_FLAGS_BOOLEAN_LIKE),
            "bigint" => t.flags.intersects(TYPE_FLAGS_BIG_INT_LIKE),
            "symbol" => t.flags.intersects(TYPE_FLAGS_ES_SYMBOL_LIKE),
            "undefined" => t.flags.contains(TypeFlags::Undefined),
            "function" => !self
                .get_signatures_of_type(t, SignatureKind::Call)
                .is_empty(),
            "object" => t.flags.contains(TypeFlags::Object) || t.flags.contains(TypeFlags::Null),
            _ => false,
        }
    }

    fn narrow_by_switch_clause(
        &mut self,
        type_: &Arc<Type>,
        flow: &Arc<FlowNode>,
        target: &FlowRef,
    ) -> Arc<Type> {
        let Some(switch_stmt) = &flow.switch_statement else {
            return Arc::clone(type_);
        };
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return Arc::clone(type_);
        };
        let discriminant = &switch_data.expression;

        let (clause_start, clause_end) = match flow.clause_range {
            Some(r) => r,
            None => {

                match (&flow.node, switch_stmt) {
                    (Some(clause), _)
                        if let NodeData::SwitchStatement(sd) = &switch_stmt.data
                            && let NodeData::CaseBlock(cb) = &sd.case_block.data
                            && let Some(idx) = cb
                                .clauses
                                .nodes
                                .iter()
                                .position(|c| Arc::ptr_eq(c, clause)) =>
                    {
                        (idx, idx + 1)
                    }
                    _ => (0, 0),
                }
            }
        };
        let range = (clause_start, clause_end);

        if self.expr_matches_target(discriminant, target) {
            return self.narrow_by_switch_on_discriminant(type_, switch_stmt, range);
        }

        if let FlowRef::Symbol(symbol) = target {
        if let Some(access) = self
            .discriminant_alias_access(discriminant, symbol)
            .or_else(|| {
                self.is_property_access_on_symbol(discriminant, symbol)
                    .then(|| Arc::clone(discriminant))
            })
        {
            return self.narrow_by_switch_on_discriminant_property(
                type_,
                switch_stmt,
                range,
                &access,
            );
        }
        }

        if discriminant.kind == SyntaxKind::TypeOfExpression {
            if let NodeData::TypeOfExpression(typeof_data) = &discriminant.data {
                if self.expr_matches_target(&typeof_data.expression, target) {
                    return self.narrow_by_switch_on_typeof(type_, switch_stmt, range);
                }
            }
        }

        if discriminant.kind == SyntaxKind::TrueKeyword {
            return self.narrow_by_switch_on_true(type_, switch_stmt, range, target);
        }

        Arc::clone(type_)
    }

    fn narrow_by_switch_on_true(
        &mut self,
        type_: &Arc<Type>,
        switch_stmt: &Arc<Node>,
        (clause_start, clause_end): (usize, usize),
        target: &FlowRef,
    ) -> Arc<Type> {
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return Arc::clone(type_);
        };
        let NodeData::CaseBlock(case_block) = &switch_data.case_block.data else {
            return Arc::clone(type_);
        };
        let clauses = &case_block.clauses.nodes;

        let has_default = clause_start == clause_end
            || clauses[clause_start..clause_end]
                .iter()
                .any(|c| c.kind == SyntaxKind::DefaultClause);

        let narrow_away = |checker: &mut Self, t: &Arc<Type>, clauses: &[Arc<Node>]| {
            let mut t = Arc::clone(t);
            for clause in clauses {
                if clause.kind == SyntaxKind::CaseClause
                    && let NodeData::CaseOrDefaultClause(cd) = &clause.data
                {
                    t = checker.narrow_by_expression(
                        &t,
                        &cd.expression,
                        target,
                        NarrowKind::FalseBranch,
                        0,
                    );
                }
            }
            t
        };

        let mut t = narrow_away(self, type_, &clauses[..clause_start.min(clauses.len())]);

        if has_default {
            let end = clause_end.min(clauses.len());
            if end < clauses.len() {
                t = narrow_away(self, &t, &clauses[end..]);
            }
            return t;
        }

        let mut parts: Vec<Arc<Type>> = Vec::new();
        for clause in &clauses[clause_start..clause_end.min(clauses.len())] {
            if clause.kind == SyntaxKind::CaseClause
                && let NodeData::CaseOrDefaultClause(cd) = &clause.data
            {
                let narrowed = self.narrow_by_expression(
                    &t,
                    &cd.expression,
                    target,
                    NarrowKind::TrueBranch,
                    0,
                );
                if !parts.iter().any(|p| Arc::ptr_eq(p, &narrowed)) {
                    parts.push(narrowed);
                }
            }
        }
        if parts.is_empty() {
            return t;
        }
        if parts.len() == 1 {
            return parts.into_iter().next().expect("exactly one");
        }
        self.get_union_type(parts)
    }

    fn narrow_by_switch_on_typeof(
        &mut self,
        type_: &Arc<Type>,
        switch_stmt: &Arc<Node>,
        (clause_start, clause_end): (usize, usize),
    ) -> Arc<Type> {
        let witnesses = self.get_switch_clause_typeof_witnesses(switch_stmt);
        let Some(witnesses) = witnesses else {
            return Arc::clone(type_);
        };
        let start = clause_start.min(witnesses.len());
        let end = clause_end.min(witnesses.len());
        let has_default = clause_start == clause_end
            || clauses_of_range(switch_stmt, clause_start, clause_end)
                .iter()
                .any(|c| c.kind == SyntaxKind::DefaultClause);
        if has_default {

            let mut outside_implied: Vec<Arc<Type>> = Vec::new();
            for (i, w) in witnesses.iter().enumerate() {
                if (i < start || i >= end) && !w.is_empty() {
                    outside_implied.push(self.typeof_string_to_type(w));
                }
            }
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| {
                    !outside_implied.iter().any(|it| self.types_overlap(t, it))
                })
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }

        let group_witnesses: Vec<(String, Arc<Type>)> = witnesses[start..end]
            .iter()
            .filter(|w| !w.is_empty())
            .map(|w| (w.clone(), self.typeof_string_to_type(w)))
            .collect();
        if group_witnesses.is_empty() {
            return Arc::clone(type_);
        }

        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let matching: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| {
                    group_witnesses.iter().any(|(text, implied)| {

                        if text == "function" {
                            return self.types_overlap(t, implied)
                                && !self
                                    .get_signatures_of_type(t, SignatureKind::Call)
                                    .is_empty();
                        }
                        self.types_overlap(t, implied)
                    })
                })
                .collect();
            return self.rebuild_union_or_never(type_, matching);
        }

        let overlapped: Vec<Arc<Type>> = group_witnesses
            .iter()
            .filter(|(_, implied)| self.types_overlap(type_, implied))
            .map(|(_, implied)| Arc::clone(implied))
            .collect();
        if overlapped.is_empty() {

            return self.never_type();
        }
        if overlapped.len() == 1 {
            let implied = overlapped.into_iter().next().expect("exactly one");
            if self.is_type_assignable_to(type_, &implied) {
                return Arc::clone(type_);
            }
            return implied;
        }
        self.get_union_type(overlapped)
    }

    fn get_switch_clause_typeof_witnesses(
        &mut self,
        switch_stmt: &Arc<Node>,
    ) -> Option<Vec<String>> {
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return None;
        };
        let NodeData::CaseBlock(case_block) = &switch_data.case_block.data else {
            return None;
        };
        let mut witnesses = Vec::with_capacity(case_block.clauses.len());
        for clause in &case_block.clauses.nodes {
            if clause.kind == SyntaxKind::CaseClause {
                if let NodeData::CaseOrDefaultClause(cd) = &clause.data {
                    let text = self.literal_text_of(&cd.expression);
                    match text {
                        Some(t) => witnesses.push(t),
                        None => return None,
                    }
                } else {
                    witnesses.push(String::new());
                }
            } else {

                witnesses.push(String::new());
            }
        }
        Some(witnesses)
    }

    fn typeof_string_to_type(&mut self, text: &str) -> Arc<Type> {
        match text {
            "string" => self.string_type(),
            "number" => self.number_type(),
            "bigint" => self.bigint_type(),
            "boolean" => self.boolean_type(),
            "symbol" => self.es_symbol_type(),
            "undefined" => self.undefined_type(),
            "object" => {

                let non_primitive = self.non_primitive_type();
                let null = self.null_type();
                self.get_union_type(vec![non_primitive, null])
            }
            "function" => {

                if let Some(f) = self.any_function_type.get() {
                    Arc::clone(f)
                } else {
                    self.any_type()
                }
            }
            _ => self.non_primitive_type(),
        }
    }

    fn literal_text_of(&self, node: &Arc<Node>) -> Option<String> {
        match node.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(data) = &node.data {
                    Some(data.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                if let NodeData::NoSubstitutionTemplateLiteral(data) = &node.data {
                    Some(data.text.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn narrow_by_switch_on_discriminant(
        &mut self,
        type_: &Arc<Type>,
        switch_stmt: &Arc<Node>,
        (clause_start, clause_end): (usize, usize),
    ) -> Arc<Type> {
        let case_types = self.get_switch_clause_types(switch_stmt);
        let group_clauses = clauses_of_range(switch_stmt, clause_start, clause_end);

        let group_case_types: Vec<Arc<Type>> = group_clauses
            .iter()
            .filter(|c| c.kind == SyntaxKind::CaseClause)
            .filter_map(|c| match &c.data {
                NodeData::CaseOrDefaultClause(cd) => {
                    Some(self.get_type_of_node(&cd.expression))
                }
                _ => None,
            })
            .collect();
        if group_case_types.is_empty() {
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !case_types.iter().any(|ct| self.types_overlap(t, ct)))
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }
        let group_union = if group_case_types.len() == 1 {
            group_case_types.into_iter().next().expect("exactly one")
        } else {
            self.get_union_type(group_case_types)
        };

        let case_part = self.intersect_or_narrow(type_, &group_union);

        let has_default_in_group = group_clauses
            .iter()
            .any(|c| c.kind == SyntaxKind::DefaultClause);
        if has_default_in_group {
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !case_types.iter().any(|ct| self.types_overlap(t, ct)))
                .collect();
            let default_part = self.rebuild_union_or_never(type_, remaining);
            return self.get_union_type(vec![case_part, default_part]);
        }
        case_part
    }

    fn narrow_by_switch_on_discriminant_property(
        &mut self,
        type_: &Arc<Type>,
        switch_stmt: &Arc<Node>,
        (clause_start, clause_end): (usize, usize),
        access: &Arc<Node>,
    ) -> Arc<Type> {
        let Some(prop_name) = Self::get_accessed_property_name_from_node(access) else {
            return Arc::clone(type_);
        };
        let group_clauses = clauses_of_range(switch_stmt, clause_start, clause_end);
        let is_default = group_clauses.is_empty()
            || group_clauses
                .iter()
                .all(|c| c.kind == SyntaxKind::DefaultClause);

        if !type_.is_union() {
            let mut any_overlap = is_default;
            for clause in &group_clauses {
                if clause.kind == SyntaxKind::DefaultClause {
                    continue;
                }
                if let NodeData::CaseOrDefaultClause(cd) = &clause.data {
                    let case_type = self.get_type_of_node(&cd.expression);
                    if let Some(prop_type) = self.get_property_type_of_type(type_, &prop_name)
                        && self.types_overlap(&prop_type, &case_type)
                    {
                        any_overlap = true;
                    }
                }
            }
            if !any_overlap {
                return self.never_type();
            }
            return Arc::clone(type_);
        }

        let group_case_types: Vec<Arc<Type>> = group_clauses
            .iter()
            .filter(|c| c.kind == SyntaxKind::CaseClause)
            .filter_map(|c| match &c.data {
                NodeData::CaseOrDefaultClause(cd) => {
                    Some(self.get_type_of_node(&cd.expression))
                }
                _ => None,
            })
            .collect();
        let all_case_types = if is_default {
            self.get_switch_clause_types(switch_stmt)
        } else {
            Vec::new()
        };
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let prop_type = self.get_property_type_of_type(t, &prop_name);
                let Some(prop_type) = prop_type else {

                    return is_default;
                };
                if is_default {

                    !all_case_types
                        .iter()
                        .any(|ct| self.types_overlap(&prop_type, ct))
                } else {

                    group_case_types
                        .iter()
                        .any(|ct| self.types_overlap(&prop_type, ct))
                }
            })
            .collect();
        self.rebuild_union_or_never(type_, filtered)
    }

    fn get_switch_clause_types(&mut self, switch_stmt: &Arc<Node>) -> Vec<Arc<Type>> {
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return Vec::new();
        };
        let NodeData::CaseBlock(case_block) = &switch_data.case_block.data else {
            return Vec::new();
        };
        let mut types = Vec::with_capacity(case_block.clauses.len());
        for clause in &case_block.clauses.nodes {
            if clause.kind == SyntaxKind::CaseClause {
                if let NodeData::CaseOrDefaultClause(cd) = &clause.data {
                    types.push(self.get_type_of_node(&cd.expression));
                    continue;
                }
            }
            types.push(self.never_type());
        }
        types
    }

    fn optional_chain_contains_target(&self, source: &Arc<Node>, target: &FlowRef) -> bool {
        let symbol = match target {
            FlowRef::Symbol(symbol) => symbol,
            FlowRef::Node(reference) => {
                return self.optional_chain_contains_reference(source, reference)
            }
        };
        self.optional_chain_contains_symbol(source, symbol)
    }

    fn optional_chain_contains_symbol(&self, source: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        let mut current = Arc::clone(source);
        loop {
            let (inner, is_optional) = match &current.data {
                NodeData::PropertyAccessExpression(pa) => {
                    (&pa.expression, pa.question_dot_token.is_some())
                }
                NodeData::ElementAccessExpression(ea) => {
                    (&ea.expression, ea.question_dot_token.is_some())
                }
                NodeData::CallExpression(ce) => (&ce.expression, ce.question_dot_token.is_some()),
                NodeData::NonNullExpression(ne) => (&ne.expression, false),
                NodeData::ParenthesizedExpression(pe) => (&pe.expression, false),
                _ => return false,
            };
            if is_optional && self.is_symbol_identifier(inner, symbol) {
                return true;
            }
            if !is_optional
                && !matches!(
                    &current.data,
                    NodeData::NonNullExpression(_) | NodeData::ParenthesizedExpression(_)
                )
            {

                return false;
            }
            current = Arc::clone(inner);
        }
    }

    fn optional_chain_contains_reference(
        &self,
        source: &Arc<Node>,
        reference: &Arc<Node>,
    ) -> bool {
        let mut current = Arc::clone(source);
        loop {
            let (inner, is_optional) = match &current.data {
                NodeData::PropertyAccessExpression(pa) => {
                    (&pa.expression, pa.question_dot_token.is_some())
                }
                NodeData::ElementAccessExpression(ea) => {
                    (&ea.expression, ea.question_dot_token.is_some())
                }
                NodeData::CallExpression(ce) => (&ce.expression, ce.question_dot_token.is_some()),
                NodeData::NonNullExpression(ne) => (&ne.expression, false),
                NodeData::ParenthesizedExpression(pe) => (&pe.expression, false),
                _ => return false,
            };
            if is_optional && self.is_matching_reference(reference, inner) {
                return true;
            }
            if !is_optional
                && !matches!(
                    &current.data,
                    NodeData::NonNullExpression(_) | NodeData::ParenthesizedExpression(_)
                )
            {

                return false;
            }
            current = Arc::clone(inner);
        }
    }

    fn narrow_by_optional_chain_containment(
        &mut self,
        type_: &Arc<Type>,
        op: SyntaxKind,
        value_node: &Arc<Node>,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let is_equality =
            op == SyntaxKind::EqualsEqualsEqualsToken || op == SyntaxKind::EqualsEqualsToken;
        let is_loose =
            op == SyntaxKind::EqualsEqualsToken || op == SyntaxKind::ExclamationEqualsToken;

        let nullable_flags = if is_loose {
            TypeFlags::Undefined | TypeFlags::Null
        } else {
            TypeFlags::Undefined
        };
        let value_type = self.get_type_of_node(value_node);

        let value_is_nullable = self.type_contains_flags(&value_type, nullable_flags);
        let value_excludes_nullable = !value_is_nullable;
        let remove_nullable = if is_equality {

            (kind == NarrowKind::TrueBranch && value_excludes_nullable)
                || (kind == NarrowKind::FalseBranch && value_is_nullable)
        } else {

            (kind == NarrowKind::FalseBranch && value_excludes_nullable)
                || (kind == NarrowKind::TrueBranch && value_is_nullable)
        };
        if remove_nullable {
            self.remove_nullable_from_union(type_)
        } else {
            Arc::clone(type_)
        }
    }

    fn remove_nullable_from_union(&self, type_: &Arc<Type>) -> Arc<Type> {
        self.remove_flags_from_union(type_, TypeFlags::Undefined | TypeFlags::Null)
    }

    fn type_contains_flags(&self, type_: &Arc<Type>, flags: TypeFlags) -> bool {
        if type_.flags.intersects(flags) {
            return true;
        }
        if type_.is_union() {
            if let TypeData::Union(u) = &type_.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|t| t.flags.intersects(flags));
            }
        }
        false
    }

    fn narrow_by_call_expression(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let NodeData::CallExpression(call) = &expr.data else {
            return Arc::clone(type_);
        };
        let callee_type = self.get_type_of_node(&call.expression);
        let signatures = self.get_signatures_of_type(&callee_type, SignatureKind::Call);
        let assume_true = kind == NarrowKind::TrueBranch;
        for sig in &signatures {
            let Some(predicate) = self.compute_type_predicate_of_signature(sig) else {
                continue;
            };

            if predicate.kind != TypePredicateKind::Identifier
                && predicate.kind != TypePredicateKind::AssertsIdentifier
            {

                if predicate.kind == TypePredicateKind::This
                    && let Some(pred_type) = &predicate.t
                {
                    let receiver = match &call.expression.data {
                        NodeData::PropertyAccessExpression(pa) => Some(&pa.expression),
                        _ => None,
                    };
                    let Some(receiver) = receiver else {
                        continue;
                    };
                    if !self.expr_matches_target(receiver, target) {
                        continue;
                    }
                    let Some(callback_arg) = call.arguments.nodes.first() else {
                        continue;
                    };
                    let Some(u) = self.callback_predicate_type(callback_arg) else {
                        continue;
                    };

                    let instantiated = if sig.type_parameters.is_empty() {
                        Arc::clone(pred_type)
                    } else {
                        let args: Vec<Arc<Type>> = sig
                            .type_parameters
                            .iter()
                            .map(|_| Arc::clone(&u))
                            .collect();
                        self.substitute_infer_type_parameters(
                            pred_type,
                            &sig.type_parameters,
                            &args,
                        )
                    };
                    return self.narrow_by_type_predicate(type_, &instantiated, assume_true);
                }
                continue;
            }
            let Some(pred_type) = &predicate.t else {
                continue;
            };
            let param_idx = predicate.parameter_index as usize;
            let Some(arg) = call.arguments.nodes.get(param_idx) else {
                continue;
            };

            if !self.expr_matches_target(arg, target) {
                continue;
            }
            return self.narrow_by_type_predicate(type_, pred_type, assume_true);
        }
        Arc::clone(type_)
    }

    fn callback_predicate_type(&mut self, arg: &Arc<Node>) -> Option<Arc<Type>> {
        let arg_type = self.get_type_of_node(arg);
        let sigs = self.get_signatures_of_type(&arg_type, SignatureKind::Call);
        for sig in &sigs {
            if let Some(pred) = self.compute_type_predicate_of_signature(sig)
                && pred.kind == TypePredicateKind::Identifier
                && let Some(t) = pred.t
            {
                return Some(t);
            }
        }
        None
    }

    fn narrow_by_assertion_call(
        &mut self,
        type_: &Arc<Type>,
        call_expr: &Arc<Node>,
        target: &FlowRef,
    ) -> Arc<Type> {
        let NodeData::CallExpression(call) = &call_expr.data else {
            return Arc::clone(type_);
        };
        let callee_type = self.get_type_of_node(&call.expression);
        let signatures = self.get_signatures_of_type(&callee_type, SignatureKind::Call);
        for sig in &signatures {
            let Some(predicate) = self.compute_type_predicate_of_signature(sig) else {
                continue;
            };

            if predicate.kind != TypePredicateKind::AssertsIdentifier
                && predicate.kind != TypePredicateKind::AssertsThis
            {
                continue;
            }

            if predicate.kind == TypePredicateKind::AssertsThis {
                continue;
            }
            let param_idx = predicate.parameter_index as usize;
            let Some(arg) = call.arguments.nodes.get(param_idx) else {
                continue;
            };

            if !self.expr_matches_target(arg, target) {

                if let Some(narrowed) =
                    self.narrow_by_asserted_comparison(type_, arg, target)
                {
                    return narrowed;
                }
                continue;
            }
            if let Some(pred_type) = &predicate.t {

                return self.intersect_or_narrow(type_, pred_type);
            }

            return self.remove_flags_from_union(type_, TYPE_FLAGS_NULLABLE);
        }
        Arc::clone(type_)
    }

    fn narrow_by_asserted_comparison(
        &mut self,
        type_: &Arc<Type>,
        arg: &Arc<Node>,
        target: &FlowRef,
    ) -> Option<Arc<Type>> {
        let NodeData::BinaryExpression(bin) = &arg.data else {
            return None;
        };
        use crate::ast::SyntaxKind::*;
        let (cmp, target_side, literal_side) = match bin.operator_token.kind {
            ExclamationEqualsEqualsToken | ExclamationEqualsToken
            | EqualsEqualsEqualsToken | EqualsEqualsToken => {
                let l_matches = self.expr_matches_target(&bin.left, target);
                let r_matches = self.expr_matches_target(&bin.right, target);
                if l_matches {
                    (bin.operator_token.kind, &bin.left, &bin.right)
                } else if r_matches {
                    (bin.operator_token.kind, &bin.right, &bin.left)
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        let _ = target_side;
        let lt = self.get_type_of_node(literal_side);
        let is_eq = matches!(cmp, EqualsEqualsEqualsToken | EqualsEqualsToken);
        if is_eq {
            Some(self.intersect_or_narrow(type_, &lt))
        } else {
            Some(self.remove_type_from_union(type_, &lt))
        }
    }

    fn narrow_by_type_predicate(
        &mut self,
        type_: &Arc<Type>,
        pred_type: &Arc<Type>,
        assume_true: bool,
    ) -> Arc<Type> {

        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(type_);
        }
        if assume_true {
            self.intersect_or_narrow(type_, pred_type)
        } else {

            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !self.is_type_assignable_to(t, pred_type))
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }
    }

    fn typeof_expr_matches_target(&self, expr: &Arc<Node>, target: &FlowRef) -> bool {
        let NodeData::TypeOfExpression(typeof_data) = &expr.data else {
            return false;
        };
        self.expr_matches_target(&typeof_data.expression, target)
    }

    fn narrow_by_typeof(
        &mut self,
        type_: &Arc<Type>,
        type_name_node: &Arc<Node>,
        narrow_to_value: bool,
        is_loose: bool,
    ) -> Arc<Type> {
        let type_name = match &type_name_node.data {
            NodeData::StringLiteral(data) => data.text.as_str(),
            _ => return Arc::clone(type_),
        };

        if let TypeData::Intersection(i) = &type_.data {
            let all_primitive = i
                .union_or_intersection
                .types
                .iter()
                .all(|t| t.flags.intersects(TYPE_FLAGS_PRIMITIVE));
            if !all_primitive {
                return Arc::clone(type_);
            }
        }
        let matching_flags = match type_name {
            "string" => TYPE_FLAGS_STRING_LIKE,
            "number" => TYPE_FLAGS_NUMBER_LIKE,
            "boolean" => TYPE_FLAGS_BOOLEAN_LIKE,
            "bigint" => TYPE_FLAGS_BIG_INT_LIKE,
            "symbol" => TYPE_FLAGS_ES_SYMBOL_LIKE,
            "undefined" => TypeFlags::Undefined,
            "function" => {

                return self.filter_type_by_callable(type_, narrow_to_value);
            }
            "object" => {

                if narrow_to_value {
                    return self.filter_type_by_object(type_, is_loose);
                }
                return self.remove_object_from_union(type_);
            }
            _ => return Arc::clone(type_),
        };
        if narrow_to_value {
            self.filter_type_by_flags(type_, matching_flags)
        } else {
            self.remove_flags_from_union(type_, matching_flags)
        }
    }

    fn narrow_by_truthiness(&self, type_: &Arc<Type>, kind: NarrowKind) -> Arc<Type> {
        match kind {
            NarrowKind::TrueBranch => {

                let falsy_flags = TypeFlags::Undefined
                    | TypeFlags::Null
                    | TypeFlags::Void
                    | TypeFlags::BooleanLiteral
                    | TypeFlags::StringLiteral
                    | TypeFlags::NumberLiteral;
                self.remove_falsy_from_union(type_, falsy_flags)
            }
            NarrowKind::FalseBranch => {

                self.filter_to_falsy(type_)
            }
        }
    }

    fn narrow_by_optionality(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
        _depth: u32,
    ) -> Arc<Type> {

        if self.expr_matches_target(expr, target) {
            return match kind {
                NarrowKind::TrueBranch => self.remove_nullable_from_union(type_),
                NarrowKind::FalseBranch => {
                    self.filter_type_by_flags(type_, TypeFlags::Undefined | TypeFlags::Null)
                }
            };
        }

        if expr.kind == SyntaxKind::Identifier && self.flow_inline_level < 5 {
            if let Some(init_expr) = self.const_alias_initializer(expr) {
                self.flow_inline_level += 1;
                let result = self.narrow_by_optionality(type_, &init_expr, target, kind, _depth);
                self.flow_inline_level -= 1;
                return result;
            }
        }

        Arc::clone(type_)
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

    fn constituent_is_definitely_falsy(&self, t: &Arc<Type>) -> bool {
        if t.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
            return true;
        }
        if t.flags.contains(TypeFlags::BooleanLiteral) {

            return matches!(t.literal_value(), Some(crate::checker::types::LiteralValue::Boolean(false)));
        }
        if t.flags.contains(TypeFlags::StringLiteral) {
            return t.intrinsic_name().is_some_and(|n| n == "\"\"" || n.is_empty());
        }
        if t.flags.contains(TypeFlags::NumberLiteral) {
            return t.intrinsic_name().is_some_and(|n| n == "0");
        }
        false
    }

    pub(super) fn flow_constituents_public(&self, t: &Arc<Type>) -> Vec<Arc<Type>> {
        self.constituent_types(t)
    }

    pub(super) fn flow_constituent_definitely_falsy(&self, t: &Arc<Type>) -> bool {
        self.constituent_is_definitely_falsy(t)
    }

    fn extract_definitely_falsy_constituents(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let falsy: Vec<Arc<Type>> = self
            .constituent_types(t)
            .into_iter()
            .filter(|c| self.constituent_is_definitely_falsy(c))
            .collect();
        self.rebuild_union_or_never(t, falsy)
    }

    fn remove_definitely_falsy_constituents(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let kept: Vec<Arc<Type>> = self
            .constituent_types(t)
            .into_iter()
            .filter(|c| !self.constituent_is_definitely_falsy(c))
            .collect();
        if kept.is_empty() {
            return Arc::clone(t);
        }
        self.rebuild_union_or_never(t, kept)
    }

    fn flow_union_of(&self, types: &[Arc<Type>]) -> Arc<Type> {
        let mut all: Vec<Arc<Type>> = Vec::new();
        for t in types {
            for c in self.constituent_types(t) {
                if !all.iter().any(|s| Arc::ptr_eq(s, &c)) {
                    all.push(c);
                }
            }
        }
        if all.is_empty() {
            return self.never_type();
        }
        if all.len() == 1 {
            return all.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: all,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    fn remove_type_from_union(&self, type_: &Arc<Type>, value_type: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| !self.types_overlap(t, value_type))
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }

        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub fn remove_flags_from_union(&self, type_: &Arc<Type>, flags: TypeFlags) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| !t.flags.intersects(flags))
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    fn filter_type_by_flags(&self, type_: &Arc<Type>, flags: TypeFlags) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| t.flags.intersects(flags))
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: matching,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    fn filter_type_by_object(&self, type_: &Arc<Type>, is_loose: bool) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let mut matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {

                t.flags.contains(TypeFlags::Object)
                    || t.flags.contains(TypeFlags::Null)
                    || (is_loose && t.flags.contains(TypeFlags::Undefined))
            })
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: matching.drain(..).collect(),
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    fn filter_type_by_callable(&self, type_: &Arc<Type>, keep_callable: bool) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let is_callable = !self
                    .get_signatures_of_type(t, SignatureKind::Call)
                    .is_empty();
                if keep_callable {
                    is_callable
                } else {
                    !is_callable
                }
            })
            .collect();
        if filtered.is_empty() {
            return self.never_type();
        }
        if filtered.len() == 1 {
            return filtered.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: filtered,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    fn remove_object_from_union(&self, type_: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| !t.flags.contains(TypeFlags::Object) && !t.flags.contains(TypeFlags::Null))
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    fn remove_falsy_from_union(&self, type_: &Arc<Type>, falsy_flags: TypeFlags) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                if t.flags.intersects(falsy_flags) {

                    if t.flags.contains(TypeFlags::BooleanLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            return matches!(lit.value, LiteralValue::Boolean(true));
                        }
                    }

                    if t.flags.contains(TypeFlags::StringLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            if let LiteralValue::String(s) = &lit.value {
                                return !s.is_empty();
                            }
                        }
                        return false;
                    }

                    if t.flags.contains(TypeFlags::NumberLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            if let LiteralValue::Number(n) = &lit.value {
                                return n.0 != 0.0;
                            }
                        }
                        return false;
                    }
                    return false;
                }
                true
            })
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    fn filter_to_falsy(&self, type_: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let falsy_flags =
            TypeFlags::Undefined | TypeFlags::Null | TypeFlags::Void | TypeFlags::BooleanLiteral;
        let matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                if t.flags.intersects(falsy_flags) {

                    if t.flags.contains(TypeFlags::BooleanLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            return matches!(lit.value, LiteralValue::Boolean(false));
                        }
                    }
                    return true;
                }

                if t.flags.contains(TypeFlags::StringLiteral) {
                    if let TypeData::Literal(lit) = &t.data {
                        if let LiteralValue::String(s) = &lit.value {
                            return s.is_empty();
                        }
                    }
                }

                if t.flags.contains(TypeFlags::NumberLiteral) {
                    if let TypeData::Literal(lit) = &t.data {
                        if let LiteralValue::Number(n) = &lit.value {
                            return n.0 == 0.0;
                        }
                    }
                }
                false
            })
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: matching,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    fn intersect_or_narrow(&mut self, type_: &Arc<Type>, value_type: &Arc<Type>) -> Arc<Type> {

        if self.is_type_assignable_to(value_type, type_) {
            return Arc::clone(value_type);
        }

        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let matching: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| self.is_type_assignable_to(value_type, t))
                .collect();
            if matching.len() == 1 {
                return matching.into_iter().next().expect("exactly one");
            }
            if matching.is_empty() {
                return Arc::clone(value_type);
            }
            return self.get_union_type(matching);
        }
        Arc::clone(value_type)
    }

    fn types_overlap(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {

        if a.flags.contains(TypeFlags::Union)
            || b.flags.contains(TypeFlags::Union)
            || a.flags.contains(TypeFlags::Intersection)
            || b.flags.contains(TypeFlags::Intersection)
        {
            let a_types = self.constituent_types(a);
            let b_types = self.constituent_types(b);
            for at in &a_types {
                for bt in &b_types {
                    if self.literals_overlap(at, bt) {
                        return true;
                    }
                }
            }
            return false;
        }
        self.literals_overlap(a, b)
    }

    fn literals_overlap(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {

        let a_is_literal = a.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::BooleanLiteral,
        );
        let b_is_literal = b.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::BooleanLiteral,
        );
        if a_is_literal && b_is_literal {

            return match (&a.data, &b.data) {
                (TypeData::Literal(a_lit), TypeData::Literal(b_lit)) => a_lit.value == b_lit.value,
                _ => false,
            };
        }
        if a_is_literal {

            return a.flags.intersects(b.flags);
        }
        if b_is_literal {
            return a.flags.intersects(b.flags);
        }

        a.flags.intersects(b.flags)
    }

    fn is_symbol_identifier(&self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {

        if matches!(
            node.kind,
            SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement
        ) {
            return self
                .program
                .symbol_map()
                .symbol_of(node)
                .is_some_and(|s| Arc::ptr_eq(s, symbol));
        }
        if node.kind != SyntaxKind::Identifier {
            return false;
        }

        let symbol_map = self.program.symbol_map();
        if let Some(sym) = symbol_map.symbol_of(node) {
            let eq = Arc::ptr_eq(sym, symbol);
            return eq;
        }

        let node_name = match &node.data {
            NodeData::Identifier(data) => &data.text,
            _ => return false,
        };
        let eq = node_name == &symbol.name;
        eq
    }

    fn expr_matches_target(&self, node: &Arc<Node>, target: &FlowRef) -> bool {
        match target {
            FlowRef::Symbol(symbol) => self.is_symbol_identifier(node, symbol),
            FlowRef::Node(reference) => self.is_matching_reference(reference, node),
        }
    }

    fn is_matching_reference(&self, source: &Arc<Node>, target: &Arc<Node>) -> bool {
        match &target.data {

            NodeData::ParenthesizedExpression(p) => {
                return self.is_matching_reference(source, &p.expression);
            }
            NodeData::NonNullExpression(n) => {
                return self.is_matching_reference(source, &n.expression);
            }
            _ => {}
        }
        match target.kind {
            SyntaxKind::BinaryExpression => {
                if let NodeData::BinaryExpression(bin) = &target.data {
                    if is_assignment_operator(bin.operator_token.kind)
                        && self.is_matching_reference(source, &bin.left)
                    {
                        return true;
                    }
                    if bin.operator_token.kind == SyntaxKind::CommaToken
                        && self.is_matching_reference(source, &bin.right)
                    {
                        return true;
                    }
                }
                return false;
            }
            _ => {}
        }
        match source.kind {
            SyntaxKind::BinaryExpression => {

                if let NodeData::BinaryExpression(bin) = &source.data {
                    if bin.operator_token.kind == SyntaxKind::CommaToken {
                        return self.is_matching_reference(&bin.right, target);
                    }
                    if is_assignment_operator(bin.operator_token.kind) {
                        return self.is_matching_reference(&bin.left, target);
                    }
                }
                return false;
            }
            SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier => {
                if target.kind == SyntaxKind::Identifier {
                    return match (
                        self.resolve_identifier(source),
                        self.resolve_identifier(target),
                    ) {
                        (Some(s), Some(t)) => Arc::ptr_eq(&s, &t),
                        _ => false,
                    };
                }

                if matches!(
                    target.kind,
                    SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement
                ) {
                    let Some(source_sym) = self.resolve_identifier(source) else {
                        return false;
                    };
                    let Some(target_sym) =
                        self.program.symbol_map().symbol_of(target).cloned()
                    else {
                        return false;
                    };

                    let source_unwrapped = source_sym
                        .export_symbol
                        .clone()
                        .unwrap_or_else(|| Arc::clone(&source_sym));
                    let target_unwrapped = target_sym
                        .export_symbol
                        .clone()
                        .unwrap_or(target_sym);
                    return Arc::ptr_eq(&source_unwrapped, &target_unwrapped);
                }
                false
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => target.kind == source.kind,
            SyntaxKind::NonNullExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::SatisfiesExpression => {
                if let Some(inner) = source.expression() {
                    self.is_matching_reference(&inner, target)
                } else {
                    false
                }
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                if let Some(source_prop_name) = self.get_accessed_property_name(source) {
                    if matches!(
                        target.kind,
                        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
                    ) {
                        if let Some(target_prop_name) = self.get_accessed_property_name(target) {
                            if target_prop_name == source_prop_name {
                                let source_receiver = source.expression();
                                let target_receiver = target.expression();
                                if let (Some(s), Some(t)) = (source_receiver, target_receiver) {
                                    return self.is_matching_reference(&s, &t);
                                }
                            }
                        }
                    }
                }

                if source.kind == SyntaxKind::ElementAccessExpression
                    && target.kind == SyntaxKind::ElementAccessExpression
                {
                    let (NodeData::ElementAccessExpression(source_ea),
                         NodeData::ElementAccessExpression(target_ea)) =
                        (&source.data, &target.data)
                    else {
                        return false;
                    };
                    if source_ea.argument_expression.kind == SyntaxKind::Identifier
                        && target_ea.argument_expression.kind == SyntaxKind::Identifier
                    {
                        let matching_args = match (
                            self.resolve_identifier(&source_ea.argument_expression),
                            self.resolve_identifier(&target_ea.argument_expression),
                        ) {
                            (Some(s), Some(t)) if Arc::ptr_eq(&s, &t) => {
                                self.symbol_is_const_variable(&s)
                                    || (self.is_parameter_or_mutable_local(&s)
                                        && !self.symbol_is_assigned(&s))
                            }
                            _ => false,
                        };
                        if matching_args {
                            let (Some(s), Some(t)) = (
                                source.expression(),
                                target.expression(),
                            ) else {
                                return false;
                            };
                            return self.is_matching_reference(&s, &t);
                        }
                    }
                }
                false
            }
            SyntaxKind::QualifiedName => {
                if let NodeData::QualifiedName(qualified) = &source.data {
                    if matches!(
                        target.kind,
                        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
                    ) {
                        if let Some(target_prop_name) = self.get_accessed_property_name(target) {
                            if qualified.right.text() == target_prop_name {
                                if let Some(t) = target.expression() {
                                    return self.is_matching_reference(&qualified.left, &t);
                                }
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn contains_matching_reference(&self, source: &Arc<Node>, target: &Arc<Node>) -> bool {
        let mut source = Arc::clone(source);
        while matches!(
            source.kind,
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
        ) {
            let Some(inner) = source.expression() else {
                break;
            };
            if self.is_matching_reference(inner, target) {
                return true;
            }
            source = Arc::clone(inner);
        }
        false
    }

    fn get_accessed_property_name(&self, access: &Arc<Node>) -> Option<String> {
        match &access.data {
            NodeData::PropertyAccessExpression(pa) => Some(pa.name.text().to_string()),
            NodeData::ElementAccessExpression(ea) => {
                match &ea.argument_expression.data {
                    NodeData::StringLiteral(s) => Some(s.text.clone()),
                    NodeData::NumericLiteral(n) => Some(n.text.clone()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn is_parameter_or_mutable_local(&self, symbol: &Arc<Symbol>) -> bool {
        symbol
            .flags
            .intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable)
    }

    fn symbol_is_assigned(&self, symbol: &Arc<Symbol>) -> bool {
        let Some(decl) = symbol.value_declaration.as_ref() else {
            return true;
        };
        let Some(container) = Self::enclosing_function_or_source_file(decl) else {
            return true;
        };
        let mut assigned = false;
        Self::scan_assignment_targets(&container, &symbol.name, &mut assigned);
        assigned
    }

    pub(super) fn enclosing_function_or_source_file(node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = Arc::clone(node);
        loop {
            if matches!(
                current.kind,
                SyntaxKind::SourceFile
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::Constructor
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
            ) {
                return Some(current);
            }
            current = Arc::clone(current.parent.as_ref()?);
        }
    }

    fn scan_assignment_targets(node: &Arc<Node>, name: &str, assigned: &mut bool) {
        if *assigned {
            return;
        }
        match &node.data {
            NodeData::BinaryExpression(bin) => {
                if is_assignment_operator(bin.operator_token.kind)
                    && bin.left.kind == SyntaxKind::Identifier
                    && bin.left.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            NodeData::PrefixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && unary.operand.kind == SyntaxKind::Identifier
                    && unary.operand.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            NodeData::PostfixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && unary.operand.kind == SyntaxKind::Identifier
                    && unary.operand.text() == name
                {
                    *assigned = true;
                    return;
                }
            }
            _ => {}
        }
        crate::ast::node_data_generated::for_each_child(node, |child| {
            Self::scan_assignment_targets(child, name, assigned);
            *assigned
        });
    }

    fn const_alias_initializer(&self, expr: &Arc<Node>) -> Option<Arc<Node>> {
        if expr.kind != SyntaxKind::Identifier {
            return None;
        }

        let sym = self.resolve_identifier(expr)?;
        if !self.symbol_is_const_variable(&sym) {
            return None;
        }
        let decl = sym.value_declaration.as_ref()?;
        if decl.kind != SyntaxKind::VariableDeclaration {
            return None;
        }
        let NodeData::VariableDeclaration(var_data) = &decl.data else {
            return None;
        };

        if var_data.type_node.is_some() {
            return None;
        }
        let init = var_data.initializer.as_ref()?;
        Some(Self::skip_parentheses(init))
    }

    pub(super) fn symbol_is_const_variable(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if let Some(parent) = &decl.parent {
                if parent.kind == SyntaxKind::VariableDeclarationList
                    && parent.flags.contains(NodeFlags::Const)
                {
                    return true;
                }
            }
        }
        false
    }

    fn skip_parentheses(node: &Arc<Node>) -> Arc<Node> {
        let mut current = Arc::clone(node);
        loop {
            if let NodeData::ParenthesizedExpression(p) = &current.data {
                current = Arc::clone(&p.expression);
                continue;
            }
            return current;
        }
    }

    fn evolve_array_at_mutation(
        &mut self,
        node: &Arc<Node>,
        pre_type: &Arc<Type>,
        target: &FlowRef,
    ) -> Option<Arc<Type>> {

        let receiver = self.get_array_mutation_receiver(node)?;
        if !self.expr_matches_target(&receiver, target) {
            return None;
        }

        let evolving = if pre_type.object_flags.contains(ObjectFlags::EvolvingArray) {
            Arc::clone(pre_type)
        } else if self.is_auto_array_type(pre_type) {
            self.get_evolving_array_type(self.never_type())
        } else {

            return Some(Arc::clone(pre_type));
        };

        let args = self.get_call_arguments(node);
        let mut arg_types: Vec<Arc<Type>> = Vec::with_capacity(args.len());
        for arg in &args {
            let t = self.get_type_of_node(arg);
            arg_types.push(self.get_widened_type_of_literal(&t));
        }

        let mut evolved = evolving;
        match &node.data {
            NodeData::BinaryExpression(bin)
                if is_assignment_operator(bin.operator_token.kind) =>
            {
                if let NodeData::ElementAccessExpression(ea) = &bin.left.data {
                    let index_type = self.get_type_of_node(&ea.argument_expression);
                    if index_type
                        .flags
                        .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
                    {
                        let t = self.get_type_of_node(&bin.right);
                        let widened = self.get_widened_type_of_literal(&t);
                        evolved = self.add_evolving_array_element_type(&evolved, widened);
                    }
                }
            }
            _ => {
                for arg_type in arg_types {
                    evolved = self.add_evolving_array_element_type(&evolved, arg_type);
                }
            }
        }
        Some(evolved)
    }

    fn get_array_mutation_receiver(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            NodeData::CallExpression(call) => {

                if let NodeData::PropertyAccessExpression(prop) = &call.expression.data {
                    return Some(Arc::clone(&prop.expression));
                }
                None
            }
            NodeData::BinaryExpression(bin) => {

                if let NodeData::ElementAccessExpression(ea) = &bin.left.data {
                    return Some(Arc::clone(&ea.expression));
                }
                None
            }
            _ => None,
        }
    }

    fn get_call_arguments(&self, node: &Arc<Node>) -> Vec<Arc<Node>> {
        match &node.data {
            NodeData::CallExpression(call) => call.arguments.iter().cloned().collect(),
            _ => Vec::new(),
        }
    }

    fn binding_element_in_var_pattern(element: &Arc<Node>) -> bool {
        let pattern = element.parent.as_ref();
        let Some(decl) = pattern.and_then(|p| p.parent.as_ref()) else {
            return false;
        };
        if decl.kind != SyntaxKind::VariableDeclaration {
            return false;
        }
        let Some(list) = decl.parent.as_ref() else {
            return false;
        };
        if list.kind != SyntaxKind::VariableDeclarationList {
            return false;
        }

        !(list
            .flags
            .intersects(crate::ast::node_flags::NodeFlags::Let)
            || list
                .flags
                .intersects(crate::ast::node_flags::NodeFlags::Const))
    }

    fn assignment_flow_type(
        &mut self,
        expr: &Arc<Node>,
        target: &FlowRef,
        declared: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let evolving = declared.object_flags.contains(ObjectFlags::EvolvingArray)
            || self.is_auto_array_type(declared);
        match &expr.data {

            NodeData::BinaryExpression(bin) => {
                if !is_assignment_operator(bin.operator_token.kind) {
                    return None;
                }
                if !self.expr_matches_target(&bin.left, target) {
                    return None;
                }

                if bin.operator_token.kind == SyntaxKind::EqualsToken {

                    let assigned = if matches!(
                        &bin.right.data,
                        NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
                    ) {
                        self.auto_array_type()
                    } else {
                        self.get_type_of_node(&bin.right)
                    };
                    return Some(self.reduced_assignment_type(declared, &assigned, evolving));
                }

                let assigned = self.get_type_of_node(&bin.right);
                let possibly_nullish = self
                    .constituent_types(declared)
                    .iter()
                    .any(|c| c.flags.intersects(TypeFlags::Undefined | TypeFlags::Null));
                let possibly_falsy = self
                    .constituent_types(declared)
                    .iter()
                    .any(|c| self.constituent_is_definitely_falsy(c));
                let possibly_truthy = self
                    .constituent_types(declared)
                    .iter()
                    .any(|c| !self.constituent_is_definitely_falsy(c));
                match bin.operator_token.kind {
                    SyntaxKind::QuestionQuestionEqualsToken if possibly_nullish => {

                        let non_null = self.get_non_nullable_type_of(declared);
                        Some(self.flow_union_of(&[non_null, assigned]))
                    }
                    SyntaxKind::BarBarEqualsToken if possibly_falsy => {

                        let truthy = self.remove_definitely_falsy_constituents(declared);
                        Some(self.flow_union_of(&[truthy, assigned]))
                    }
                    SyntaxKind::AmpersandAmpersandEqualsToken if possibly_truthy => {

                        let falsy = self.extract_definitely_falsy_constituents(declared);
                        Some(self.flow_union_of(&[falsy, assigned]))
                    }
                    SyntaxKind::QuestionQuestionEqualsToken
                    | SyntaxKind::BarBarEqualsToken
                    | SyntaxKind::AmpersandAmpersandEqualsToken => {
                        Some(Arc::clone(declared))
                    }
                    _ => None,
                }
            }

            NodeData::PostfixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && self.expr_matches_target(&unary.operand, target)
                {
                    Some(self.number_type())
                } else {
                    None
                }
            }
            NodeData::PrefixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) && self.expr_matches_target(&unary.operand, target)
                {
                    Some(self.number_type())
                } else {
                    None
                }
            }

            NodeData::VariableDeclaration(_) | NodeData::BindingElement(_) => {
                let FlowRef::Symbol(symbol) = target else {
                    return None;
                };
                let element_symbol = self.program.symbol_map().symbol_of(expr).cloned();
                let matched = match &element_symbol {
                    Some(s) => {
                        Arc::ptr_eq(s, symbol)
                            || symbol
                                .export_symbol
                                .as_ref()
                                .is_some_and(|e| Arc::ptr_eq(s, e))
                    }

                    None => match &expr.data {
                        NodeData::BindingElement(be) => be
                            .name
                            .as_ref()
                            .and_then(|name| self.resolve_identifier(name))
                            .is_some_and(|s| Arc::ptr_eq(&s, symbol)),
                        _ => false,
                    },
                } || (

                    element_symbol.as_ref().is_some_and(|s| {
                        s.name == symbol.name
                            && symbol
                                .flags
                                .contains(crate::ast::SymbolFlags::FunctionScopedVariable)
                            && Self::binding_element_in_var_pattern(expr)
                    })
                );
                if !matched {
                    return None;
                }
                let assigned = self.initial_type_of_declaration(expr)?;
                Some(self.reduced_assignment_type(declared, &assigned, evolving))
            }

            NodeData::Identifier(_) if self.expr_matches_target(expr, target) => {
                Some(Arc::clone(declared))
            }
            _ => None,
        }
    }

    fn reduced_assignment_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
        evolving: bool,
    ) -> Arc<Type> {
        if evolving {
            return Arc::clone(assigned);
        }

        if declared.flags.contains(TypeFlags::Null)
            && (self.is_auto_array_type(assigned) || assigned.object_flags.contains(ObjectFlags::EvolvingArray))
        {
            return Arc::clone(assigned);
        }
        if !declared.is_union() {
            return Arc::clone(declared);
        }
        self.get_assignment_reduced_type(declared, assigned)
    }

    fn get_assignment_reduced_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
    ) -> Arc<Type> {
        if Arc::ptr_eq(declared, assigned) {
            return Arc::clone(declared);
        }
        if assigned.flags.contains(TypeFlags::Never) {
            return Arc::clone(assigned);
        }
        let constituents = self.constituent_types(declared);
        let kept: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| self.type_maybe_assignable_to(assigned, t))
            .collect();
        let reduced = self.rebuild_union_or_never(declared, kept);
        if self.is_type_assignable_to(assigned, &reduced) {
            reduced
        } else {
            Arc::clone(declared)
        }
    }

    fn type_maybe_assignable_to(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool {
        if !source.is_union() {
            return self.is_type_assignable_to(source, target);
        }
        let constituents = self.constituent_types(source);
        if constituents.iter().any(|t| Arc::ptr_eq(t, target)) {
            return true;
        }
        constituents
            .iter()
            .any(|t| self.is_type_assignable_to(t, target))
    }

    pub(super) fn initial_type_of_declaration(&mut self, expr: &Arc<Node>) -> Option<Arc<Type>> {
        match &expr.data {
            NodeData::VariableDeclaration(vd) => {
                if let Some(init) = &vd.initializer {

                    if matches!(
                        &init.data,
                        NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
                    ) {
                        return Some(self.auto_array_type());
                    }
                    if matches!(
                        init.kind,
                        crate::ast::SyntaxKind::NullKeyword | crate::ast::SyntaxKind::UndefinedKeyword
                    ) {
                        return Some(self.auto_type());
                    }
                    return Some(self.get_type_of_node(init));
                }
                let for_stmt = Self::for_in_or_of_statement_of(expr)?;
                let NodeData::ForInOrOfStatement(data) = &for_stmt.data else {
                    return None;
                };
                match for_stmt.kind {
                    SyntaxKind::ForInStatement => Some(self.string_type()),
                    SyntaxKind::ForOfStatement => {
                        let rhs = self.get_type_of_node(&data.expression);
                        Some(self.iterated_element_type(&rhs))
                    }
                    _ => None,
                }
            }
            NodeData::BindingElement(be) => {
                let pattern = Arc::clone(expr.parent.as_ref()?);
                let pattern_parent = Arc::clone(pattern.parent.as_ref()?);
                let parent_type = self.initial_type_of_declaration(&pattern_parent);
                let mut t = match (&parent_type, pattern.kind) {
                    (Some(parent_type), SyntaxKind::ObjectBindingPattern) => {
                        match Self::binding_element_property_name(expr) {
                            Some(name) => self.get_property_type_of_type(parent_type, &name),
                            None => None,
                        }
                    }
                    (
                        Some(parent_type),
                        SyntaxKind::ArrayBindingPattern,
                    ) if be.dot_dot_dot_token.is_none() => {
                        match Self::binding_element_index(&pattern, expr) {
                            Some(index) => {
                                self.destructured_array_element_type(parent_type, index)
                            }
                            None => None,
                        }
                    }
                    _ => None,
                };
                if let Some(default_expr) = &be.initializer {
                    let default_type = self.get_type_of_node(default_expr);

                    t = match t {
                        Some(t) => {
                            let non_undefined = self.remove_flags_from_union(&t, TypeFlags::Undefined);
                            Some(self.get_union_type(vec![non_undefined, default_type]))
                        }
                        None => Some(default_type),
                    };
                }
                t
            }
            _ => None,
        }
    }

    fn binding_element_property_name(element: &Arc<Node>) -> Option<String> {
        let NodeData::BindingElement(be) = &element.data else {
            return None;
        };
        if let Some(pn) = &be.property_name {
            return Some(pn.text().to_string());
        }
        be.name.as_ref().map(|n| n.text().to_string())
    }

    fn binding_element_index(pattern: &Arc<Node>, element: &Arc<Node>) -> Option<usize> {
        let NodeData::BindingPattern(data) = &pattern.data else {
            return None;
        };
        data.elements
            .nodes
            .iter()
            .position(|e| Arc::ptr_eq(e, element))
    }

    fn destructured_array_element_type(
        &mut self,
        parent_type: &Arc<Type>,
        index: usize,
    ) -> Option<Arc<Type>> {
        if self.is_tuple_type(parent_type) {
            return self.get_tuple_element_type(parent_type, index);
        }
        if self.is_array_type(parent_type) {
            return Some(self.get_array_element_type(parent_type));
        }
        Some(self.get_any_type())
    }

    fn iterated_element_type(&mut self, rhs: &Arc<Type>) -> Arc<Type> {

        if rhs.is_union() {
            let parts: Vec<Arc<Type>> = self
                .constituent_types(rhs)
                .into_iter()
                .map(|c| self.iterated_element_type(&c))
                .filter(|t| !t.flags.contains(TypeFlags::Never))
                .collect();
            if parts.is_empty() {
                return self.get_any_type();
            }
            if parts.len() == 1 {
                return parts.into_iter().next().expect("exactly one");
            }
            return self.get_union_type(parts);
        }
        if self.is_array_type(rhs) {
            return self.get_array_element_type(rhs);
        }
        if rhs.flags.intersects(TypeFlags::String | TypeFlags::StringLiteral) {
            return self.string_type();
        }
        self.get_any_type()
    }

    fn for_in_or_of_statement_of(decl: &Arc<Node>) -> Option<Arc<Node>> {
        let list = decl.parent.as_ref()?;
        if list.kind != SyntaxKind::VariableDeclarationList {
            return None;
        }
        let stmt = list.parent.as_ref()?;
        if matches!(
            stmt.kind,
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
        ) {
            Some(Arc::clone(stmt))
        } else {
            None
        }
    }

    fn for_in_expression_of(decl: &Arc<Node>) -> Option<Arc<Node>> {
        let stmt = Self::for_in_or_of_statement_of(decl)?;
        if stmt.kind != SyntaxKind::ForInStatement {
            return None;
        }
        match &stmt.data {
            NodeData::ForInOrOfStatement(d) => Some(Arc::clone(&d.expression)),
            _ => None,
        }
    }

    pub(super) fn get_property_of_type(&self, t: &Arc<Type>, name: &str) -> Option<Arc<Symbol>> {

        if let TypeData::Mapped(m) = &t.data
            && m.type_parameter.is_some()
        {
            let sym = Symbol::new(SymbolFlags::Property, name.to_string());
            return Some(Arc::new(sym));
        }

        if let Some(structured) = t.as_structured() {
            if let Some(sym) = structured.members.get(name) {
                return Some(Arc::clone(sym));
            }
        }

        let is_array_like = self.is_array_type(t)
            || matches!(&t.data, TypeData::EvolvingArray(_));
        if is_array_like
            && let Some(array_sym) = self.globals.get("Array")
        {

            if let Some(declared) = self
                .type_alias_links
                .get(array_sym)
                .and_then(|l| l.declared_type.clone())
                && let Some(structured) = declared.as_structured()
                && let Some(member) = structured.members.get(name)
            {
                return Some(Arc::clone(member));
            }

            if let Some(member) = array_sym.members.get(name) {
                return Some(Arc::clone(member));
            }
        }

        if t.flags.contains(TypeFlags::Object)
            && t.object_flags.contains(ObjectFlags::Anonymous)
            && let Some(structured) = t.as_structured()
            && structured.call_signature_count > 0
            && !self.is_array_type(t)
            && !matches!(&t.data, TypeData::EvolvingArray(_))
        {
            if let Some(function_sym) = self.globals.get("Function") {
                if let Some(member) = function_sym.members.get(name) {
                    return Some(Arc::clone(member));
                }
            }
        }

        if let Some(interface_name) = self.primitive_interface_name(t) {
            if let Some(sym) = self.globals.get(interface_name) {
                if let Some(member) = sym.members.get(name) {
                    return Some(Arc::clone(member));
                }
            }
        }
        None
    }

    fn primitive_interface_name(&self, t: &Arc<Type>) -> Option<&'static str> {
        if t.flags
            .intersects(TypeFlags::String | TypeFlags::StringLiteral)
        {
            Some("String")
        } else if t
            .flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
        {
            Some("Number")
        } else if t
            .flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            Some("Boolean")
        } else if t
            .flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            Some("BigInt")
        } else {
            None
        }
    }

    pub(super) fn get_property_type_of_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> {
        let sym = self.get_property_of_type(t, name)?;
        Some(self.get_type_of_symbol(&sym))
    }

    fn type_has_property(&self, t: &Arc<Type>, name: &str) -> PropertyPresence {
        if let Some(structured) = t.as_structured() {
            if let Some(sym) = structured.members.get(name) {
                if sym.flags.contains(SymbolFlags::Optional) {
                    return PropertyPresence::Maybe;
                }
                return PropertyPresence::Definitely;
            }
            if !structured.index_infos.is_empty() {
                return PropertyPresence::Maybe;
            }
            return PropertyPresence::DefinitelyNot;
        }

        if t.flags.contains(TypeFlags::Object) {
            return PropertyPresence::Maybe;
        }

        PropertyPresence::DefinitelyNot
    }

    fn get_instance_type_of_constructor(&mut self, ctor_type: &Arc<Type>) -> Option<Arc<Type>> {

        if let Some(prop_sym) = self.get_property_of_type(ctor_type, "prototype") {
            let prop_type = self.get_type_of_symbol(&prop_sym);
            if !prop_type.flags.contains(TypeFlags::Any) {
                return Some(prop_type);
            }
        }

        let construct_sigs = self.get_signatures_of_type(ctor_type, SignatureKind::Construct);
        if !construct_sigs.is_empty() {
            let mut return_types: Vec<Arc<Type>> = Vec::new();
            for sig in &construct_sigs {
                if let Some(rt) = self.get_return_type_of_signature(sig) {
                    if !return_types.iter().any(|t| Arc::ptr_eq(t, &rt)) {
                        return_types.push(rt);
                    }
                }
            }
            if !return_types.is_empty() {
                return Some(self.get_union_type(return_types));
            }
        }
        None
    }

    fn get_accessed_property_name_from_node(node: &Arc<Node>) -> Option<String> {
        match &node.data {
            NodeData::StringLiteral(s) => Some(s.text.clone()),
            NodeData::NumericLiteral(n) => Some(n.text.clone()),
            NodeData::Identifier(id) => Some(id.text.clone()),
            NodeData::PropertyAccessExpression(pa) => Some(pa.name.text().to_string()),
            NodeData::ElementAccessExpression(ea) => {
                Self::get_accessed_property_name_from_node(&ea.argument_expression)
            }

            NodeData::BindingElement(be) => be
                .property_name
                .as_ref()
                .map(|pn| pn.text().to_string())
                .or_else(|| be.name.as_ref().map(|n| n.text().to_string())),
            _ => None,
        }
    }

    fn discriminant_alias_access(
        &self,
        expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Node>> {
        if expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(expr)?;
        if !self.symbol_is_const_variable(&sym) {
            return None;
        }
        let decl = Arc::clone(sym.value_declaration.as_ref()?);

        if let Some(init) = Self::candidate_variable_declaration_initializer(&decl) {
            if matches!(
                init.kind,
                SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
            ) {
                if let Some(recv) = init.expression() {
                    if self.is_symbol_identifier(recv, symbol) {
                        return Some(init);
                    }
                }
            }
        }

        if decl.kind == SyntaxKind::BindingElement {
            let NodeData::BindingElement(be) = &decl.data else {
                return None;
            };
            if be.dot_dot_dot_token.is_none() && be.initializer.is_none() {
                let pattern = decl.parent.as_ref()?;
                let var_decl = Arc::clone(pattern.parent.as_ref()?);
                if let Some(init) = Self::candidate_variable_declaration_initializer(&var_decl) {
                    let init_matches = match init.kind {
                        SyntaxKind::Identifier => self.is_symbol_identifier(&init, symbol),
                        SyntaxKind::PropertyAccessExpression
                        | SyntaxKind::ElementAccessExpression => init
                            .expression()
                            .is_some_and(|recv| self.is_symbol_identifier(recv, symbol)),
                        _ => false,
                    };
                    if init_matches {
                        return Some(decl);
                    }
                }
            }
        }
        None
    }

    fn candidate_variable_declaration_initializer(decl: &Arc<Node>) -> Option<Arc<Node>> {
        let NodeData::VariableDeclaration(data) = &decl.data else {
            return None;
        };
        if data.type_node.is_some() {
            return None;
        }
        data.initializer.as_ref().map(Self::skip_parentheses)
    }

    fn is_property_access_on_reference(&self, node: &Arc<Node>, reference: &Arc<Node>) -> bool {
        let mut r = reference;
        loop {
            match &r.data {
                NodeData::ParenthesizedExpression(p) => r = &p.expression,
                NodeData::NonNullExpression(n) => r = &n.expression,
                _ => break,
            }
        }
        match &node.data {
            NodeData::PropertyAccessExpression(pa) => {
                self.is_matching_reference(r, &pa.expression)
            }
            NodeData::ElementAccessExpression(ea) => {
                self.is_matching_reference(r, &ea.expression)
            }
            _ => false,
        }
    }

    fn is_property_access_on_symbol(&self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        match &node.data {
            NodeData::PropertyAccessExpression(pa) => {

                pa.question_dot_token.is_none() && self.is_symbol_identifier(&pa.expression, symbol)
            }
            NodeData::ElementAccessExpression(ea) => {
                ea.question_dot_token.is_none() && self.is_symbol_identifier(&ea.expression, symbol)
            }
            _ => false,
        }
    }

    fn narrow_to_subtype(&mut self, type_: &Arc<Type>, candidate: &Arc<Type>) -> Arc<Type> {

        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(candidate);
        }
        if type_.is_union() {

            let constituents = self.constituent_types(type_);
            let mapped: Vec<Arc<Type>> = constituents
                .into_iter()
                .map(|t| {
                    if self.is_type_assignable_to(&t, candidate) {
                        t
                    } else if self.is_type_assignable_to(candidate, &t) {
                        Arc::clone(candidate)
                    } else {
                        self.never_type()
                    }
                })
                .collect();
            return self.rebuild_union_or_never(type_, mapped);
        }

        if self.is_type_assignable_to(candidate, type_) {
            Arc::clone(candidate)
        } else {
            Arc::clone(type_)
        }
    }

    fn remove_subtype_from_union(&mut self, type_: &Arc<Type>, candidate: &Arc<Type>) -> Arc<Type> {
        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !self.is_type_assignable_to(t, candidate))
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }
        if self.is_type_assignable_to(type_, candidate) {
            self.never_type()
        } else {
            Arc::clone(type_)
        }
    }

    fn rebuild_union_or_never(
        &mut self,
        original: &Arc<Type>,
        constituents: Vec<Arc<Type>>,
    ) -> Arc<Type> {
        if constituents.is_empty() {
            return self.never_type();
        }
        if constituents.len() == 1 {
            return constituents.into_iter().next().expect("exactly one");
        }

        if let TypeData::Union(u) = &original.data {
            if u.union_or_intersection.types.len() == constituents.len()
                && u.union_or_intersection
                    .types
                    .iter()
                    .zip(constituents.iter())
                    .all(|(a, b)| Arc::ptr_eq(a, b))
            {
                return Arc::clone(original);
            }
        }
        self.get_union_type(constituents)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PropertyPresence {

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

fn is_assignment_operator(kind: SyntaxKind) -> bool {
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

fn clauses_of_range(switch_stmt: &Arc<Node>, start: usize, end: usize) -> Vec<Arc<Node>> {
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
