#![allow(unused_imports)]

use crate::checker::flow_impl_chunk::*;
use crate::checker::utilities_token_is_identifier_or_keyword::is_unit_type;
use tsox_frontend::ast::{ModifierFlags, NodeData, SyntaxKind};

impl Checker {
    pub fn is_reachable_flow_node(&mut self, flow: &Arc<FlowNode>) -> bool {
        let mut reduce_labels: Vec<(usize, Vec<Arc<FlowNode>>)> = Vec::new();
        self.is_reachable_flow_node_worker(flow, false, &mut reduce_labels)
    }

    fn is_reachable_flow_node_worker(
        &mut self,
        flow: &Arc<FlowNode>,
        no_cache_check: bool,
        reduce_labels: &mut Vec<(usize, Vec<Arc<FlowNode>>)>,
    ) -> bool {
        let mut flow = Arc::clone(flow);
        let mut no_cache_check = no_cache_check;
        loop {
            let flags = flow.flags;
            if flags.contains(FlowFlags::SHARED) {
                if !no_cache_check && reduce_labels.is_empty() {
                    let key = Arc::as_ptr(&flow) as usize as u64;
                    if let Some(&reachable) = self.flow_node_reachable.get(&key) {
                        return reachable;
                    }
                    let reachable =
                        self.is_reachable_flow_node_worker(&flow, true, reduce_labels);
                    self.flow_node_reachable.insert(key, reachable);
                    return reachable;
                }
                no_cache_check = false;
            }
            if flags.intersects(
                FlowFlags::ASSIGNMENT
                    | FlowFlags::CONDITION
                    | FlowFlags::ARRAY_MUTATION
                    | FlowFlags::CALL,
            ) {
                let Some(antecedent) = &flow.antecedent else {
                    return false;
                };
                flow = Arc::clone(antecedent);
                continue;
            }
            if flags.contains(FlowFlags::BRANCH_LABEL) {
                let antecedents: Vec<Arc<FlowNode>> =
                    branch_label_antecedents(&flow, reduce_labels).to_vec();
                for antecedent in &antecedents {
                    if self.is_reachable_flow_node_worker(antecedent, false, reduce_labels) {
                        return true;
                    }
                }
                return false;
            }
            if flags.contains(FlowFlags::LOOP_LABEL) {
                if flow.antecedents.is_empty() {
                    return false;
                }
                flow = Arc::clone(&flow.antecedents[0]);
                continue;
            }
            if flags.contains(FlowFlags::SWITCH_CLAUSE) {
                let bypass = matches!(flow.clause_range, Some((start, end)) if start == end);
                if bypass {
                    let Some(switch_statement) = &flow.switch_statement else {
                        return false;
                    };
                    if self.is_exhaustive_switch_statement(switch_statement) {
                        return false;
                    }
                }
                let Some(antecedent) = &flow.antecedent else {
                    return false;
                };
                flow = Arc::clone(antecedent);
                continue;
            }
            if flags.contains(FlowFlags::REDUCE_LABEL) {
                let Some(target) = &flow.reduce_target else {
                    return false;
                };
                let target_key = Arc::as_ptr(target) as usize;
                let antecedents = flow.antecedents.clone();
                let Some(antecedent) = flow.antecedent.clone() else {
                    return false;
                };
                reduce_labels.push((target_key, antecedents));
                let result =
                    self.is_reachable_flow_node_worker(&antecedent, false, reduce_labels);
                reduce_labels.pop();
                return result;
            }
            return !flags.contains(FlowFlags::UNREACHABLE);
        }
    }

    pub fn is_exhaustive_switch_statement(&mut self, node: &Arc<Node>) -> bool {
        let key = Arc::as_ptr(node) as usize as u64;
        match self.switch_exhaustive_state.get(&key) {
            Some(&2) => return true,
            Some(&3) => return false,
            Some(&1) => return false,
            _ => {}
        }
        self.switch_exhaustive_state.insert(key, 1);
        let exhaustive = self.compute_exhaustive_switch_statement(node);
        self.switch_exhaustive_state
            .insert(key, if exhaustive { 2 } else { 3 });
        exhaustive
    }

    fn compute_exhaustive_switch_statement(&mut self, node: &Arc<Node>) -> bool {
        let NodeData::SwitchStatement(data) = &node.data else {
            return false;
        };
        if data.expression.kind == SyntaxKind::TypeOfExpression {
            return false;
        }
        let expression = Arc::clone(&data.expression);
        let expression_type = self.get_type_of_node(&expression);
        let t = self.get_base_constraint_or_type(&expression_type);
        if !is_literal_switch_type(&t) {
            return false;
        }
        let switch_types = self.get_switch_clause_types(node);
        if switch_types.is_empty()
            || switch_types
                .iter()
                .any(|ct| !is_unit_type(ct) && !ct.flags.contains(TypeFlags::Never))
        {
            return false;
        }
        let t = self.get_regular_type_of_literal_type(&t);
        let switch_types: Vec<Arc<Type>> = switch_types
            .iter()
            .map(|st| self.get_regular_type_of_literal_type(st))
            .collect();
        if t.flags.contains(TypeFlags::Boolean) {
            return switch_types.iter().any(|st| is_boolean_literal(st, true))
                && switch_types
                    .iter()
                    .any(|st| is_boolean_literal(st, false));
        }
        let constituents = self.constituent_types(&t);
        constituents
            .iter()
            .all(|c| switch_types.iter().any(|st| same_unit_type(c, st)))
    }

    pub fn function_has_implicit_return(&mut self, fn_node: &Arc<Node>) -> bool {
        let Some(end_flow) = self
            .program
            .symbol_map()
            .flow_node_of(fn_node)
            .map(Arc::clone)
        else {
            return false;
        };
        self.is_reachable_flow_node(&end_flow)
    }

    pub fn check_no_implicit_returns(&mut self, node: &Arc<Node>, type_node: Option<&Arc<Node>>) {
        if !self.compiler_options.no_implicit_returns.is_true() {
            return;
        }
        let Some(body) = function_like_body(node) else {
            return;
        };
        if body.kind != SyntaxKind::Block {
            return;
        }
        if !self.function_has_implicit_return(node) {
            return;
        }
        let has_explicit_return = Self::function_body_has_explicit_return(&body);
        if type_node.is_some() {
            let unwrapped = declared_unwrapped_return_type(self, node, type_node);
            self.report_all_code_paths_error(node, type_node, &unwrapped, has_explicit_return);
            return;
        }
        if !has_explicit_return {
            return;
        }
        let inferred = self.infer_function_return_type(Some(node), Some(&body), None);
        let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
        let unwrapped = self.unwrap_async_return_type(inferred, is_async);
        if self.maybe_type_of_kind(&unwrapped, TypeFlags::Void)
            || unwrapped
                .flags
                .intersects(TypeFlags::Any | TypeFlags::Undefined)
        {
            return;
        }
        let error_loc = function_like_name_loc(node).unwrap_or(node.loc);

        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            error_loc,
            tsox_core::diagnostics::messages_generated::NOT_ALL_CODE_PATHS_RETURN_A_VALUE,
            vec![],
        ));
    }

    pub fn check_all_code_paths_annotated(
        &mut self,
        node: &Arc<Node>,
        type_node: &Arc<Node>,
    ) {
        let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
        let raw = self.get_type_from_type_node(type_node);
        let unwrapped = self.unwrap_async_return_type(raw, is_async);
        if self.maybe_type_of_kind(&unwrapped, TypeFlags::Void)
            || unwrapped
                .flags
                .intersects(TypeFlags::Any | TypeFlags::Undefined)
        {
            return;
        }
        if !self.function_has_implicit_return(node) {
            return;
        }
        let has_explicit_return = function_like_body(node)
            .map(|body| Self::function_body_has_explicit_return(&body))
            .unwrap_or(false);
        self.report_all_code_paths_error(node, Some(type_node), &unwrapped, has_explicit_return);
    }

    pub(crate) fn report_all_code_paths_error(
        &mut self,
        node: &Arc<Node>,
        type_node: Option<&Arc<Node>>,
        unwrapped: &Arc<Type>,
        has_explicit_return: bool,
    ) {
        let error_loc = type_node.map_or(node.loc, |tn| tn.loc);
        if !has_explicit_return {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                error_loc,
                tsox_core::diagnostics::messages_generated::
                    A_FUNCTION_WHOSE_DECLARED_TYPE_IS_NEITHER_UNDEFINED_VOID_NOR_ANY_MUST_RETURN_A_VALUE,
                vec![],
            ));
            return;
        }
        if self.strict_null_checks {
            let undefined_type = self.undefined_type();
            if !self.is_type_assignable_to(&undefined_type, unwrapped) {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    error_loc,
                    tsox_core::diagnostics::messages_generated::
                        FUNCTION_LACKS_ENDING_RETURN_STATEMENT_AND_RETURN_TYPE_DOES_NOT_INCLUDE_UNDEFINED,
                    vec![],
                ));
                return;
            }
        }
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            error_loc,
            tsox_core::diagnostics::messages_generated::NOT_ALL_CODE_PATHS_RETURN_A_VALUE,
            vec![],
        ));
    }
}

fn branch_label_antecedents<'a>(
    flow: &'a Arc<FlowNode>,
    reduce_labels: &'a [(usize, Vec<Arc<FlowNode>>)],
) -> &'a [Arc<FlowNode>] {
    let mut i = reduce_labels.len();
    while i != 0 {
        i -= 1;
        if reduce_labels[i].0 == Arc::as_ptr(flow) as usize {
            return &reduce_labels[i].1;
        }
    }
    &flow.antecedents
}

fn is_literal_switch_type(t: &Arc<Type>) -> bool {
    if t.flags.contains(TypeFlags::Boolean) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        return t.flags.contains(TypeFlags::EnumLiteral) || constituent_flags_all_unit(t);
    }
    is_unit_type(t)
}

fn constituent_flags_all_unit(t: &Arc<Type>) -> bool {
    match &t.data {
        TypeData::Union(u) => u
            .union_or_intersection
            .types
            .iter()
            .all(|ct| is_unit_type(ct)),
        _ => false,
    }
}

fn declared_unwrapped_return_type(
    checker: &mut Checker,
    node: &Arc<Node>,
    type_node: Option<&Arc<Node>>,
) -> Arc<Type> {
    let Some(tn) = type_node else {
        return checker.get_any_type();
    };
    let raw = checker.get_type_from_type_node(tn);
    let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
    checker.unwrap_async_return_type(raw, is_async)
}

pub fn function_like_body(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.body.clone(),
        NodeData::FunctionExpression(d) => Some(d.body.clone()),
        NodeData::ArrowFunction(d) => Some(d.body.clone()),
        NodeData::MethodDeclaration(d) => d.body.clone(),
        NodeData::ConstructorDeclaration(d) => d.body.clone(),
        NodeData::GetAccessorDeclaration(d) => d.body.clone(),
        NodeData::SetAccessorDeclaration(d) => d.body.clone(),
        _ => None,
    }
}

fn same_unit_type(a: &Arc<Type>, b: &Arc<Type>) -> bool {
    if Arc::ptr_eq(a, b) {
        return true;
    }
    match (&a.data, &b.data) {
        (TypeData::Literal(la), TypeData::Literal(lb)) => la.value == lb.value,
        _ => false,
    }
}

fn function_like_name_loc(node: &Arc<Node>) -> Option<tsox_core::core::text::TextRange> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.name.as_ref().map(|n| n.loc),
        NodeData::MethodDeclaration(d) => Some(d.name.loc),
        NodeData::ArrowFunction(_) | NodeData::FunctionExpression(_) => {
            let has_async = node.has_syntactic_modifier(ModifierFlags::Async);
            has_async.then(|| {
                tsox_core::core::text::TextRange::new(node.loc.pos() - "async ".len(), node.loc.end())
            })
        }
        _ => None,
    }
}

fn is_boolean_literal(t: &Arc<Type>, value: bool) -> bool {
    matches!(
        &t.data,
        TypeData::Literal(lit) if lit.value == LiteralValue::Boolean(value)
    )
}
