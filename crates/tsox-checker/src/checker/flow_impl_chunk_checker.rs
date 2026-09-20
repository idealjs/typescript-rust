#![allow(unused_imports)]

use crate::checker::flow_impl_chunk::*;

impl Checker {
    pub fn get_narrowed_type_of_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        flow: Option<&Arc<FlowNode>>,
    ) -> Arc<Type> {
        let declared = self.get_type_of_symbol(symbol);
        self.get_narrowed_type_of_symbol_with_declared(symbol, flow, declared)
    }

    pub fn get_narrowed_type_of_symbol_with_declared(
        &mut self,
        symbol: &Arc<Symbol>,
        flow: Option<&Arc<FlowNode>>,
        declared: Arc<Type>,
    ) -> Arc<Type> {
        let frame_type = self
            .logical_rhs_narrowing_frames
            .iter()
            .rev()
            .find(|(s, _)| Arc::ptr_eq(s, symbol))
            .map(|(_, t)| Arc::clone(t));
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

    pub fn get_narrowable_type_for_reference(
        &mut self,
        t: &Arc<Type>,
        node: &Arc<Node>,
    ) -> Arc<Type> {
        if self.narrowable_reference_query_stack.len() >= 4 {
            return Arc::clone(t);
        }
        if t.flags.intersects(TypeFlags::Any | TypeFlags::Unknown) {
            return Arc::clone(t);
        }
        let generic_with_union = self.has_generic_with_union_constraint(t);
        if !generic_with_union {
            return Arc::clone(t);
        }
        if self.is_constraint_position_for_reference(t, node)
            || self.has_contextual_type_with_no_generic_types(node)
        {
            let constituents = self.constituent_types(t);
            let mapped: Vec<Arc<Type>> = constituents
                .into_iter()
                .map(|c| {
                    self.get_base_constraint_of_type(&c).unwrap_or(c)
                })
                .collect();
            if mapped.len() == 1 {
                return mapped.into_iter().next().expect("exactly one");
            }
            return self.get_union_type(mapped);
        }
        Arc::clone(t)
    }

    fn has_generic_with_union_constraint(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Intersection) {
            if let crate::checker::types::TypeData::Intersection(i) = &t.data {
                return i
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|c| self.has_generic_with_union_constraint(c));
            }
            return false;
        }
        t.flags.intersects(TYPE_FLAGS_INSTANTIABLE)
            && self
                .get_base_constraint_of_type(t)
                .is_some_and(|c| {
                    c.flags.intersects(TypeFlags::Union | TypeFlags::Null | TypeFlags::Undefined)
                })
    }

    fn is_constraint_position_for_reference(&mut self, t: &Arc<Type>, node: &Arc<Node>) -> bool {
        let Some(parent) = node.parent() else {
            return false;
        };
        match parent.kind {
            SyntaxKind::PropertyAccessExpression | SyntaxKind::QualifiedName => true,
            SyntaxKind::CallExpression | SyntaxKind::NewExpression => match &parent.data {
                tsox_frontend::ast::NodeData::CallExpression(d) => {
                    Arc::ptr_eq(&d.expression, node)
                }
                tsox_frontend::ast::NodeData::NewExpression(d) => Arc::ptr_eq(&d.expression, node),
                _ => false,
            },
            SyntaxKind::ElementAccessExpression => {
                let is_expr = match &parent.data {
                    tsox_frontend::ast::NodeData::ElementAccessExpression(d) => {
                        Arc::ptr_eq(&d.expression, node)
                    }
                    _ => false,
                };
                if !is_expr {
                    return false;
                }
                let generic_object = self.constituent_types(t).iter().all(|c| {
                    c.flags.intersects(TYPE_FLAGS_INSTANTIABLE)
                        && !self
                            .get_base_constraint_of_type(c)
                            .is_some_and(|bc| {
                                bc.flags.intersects(TypeFlags::Null | TypeFlags::Undefined)
                            })
                });
                if !generic_object {
                    return true;
                }
                let arg_generic = match &parent.data {
                    tsox_frontend::ast::NodeData::ElementAccessExpression(d) => self
                        .get_type_of_node(&d.argument_expression)
                        .flags
                        .intersects(TYPE_FLAGS_INSTANTIABLE),
                    _ => false,
                };
                !arg_generic
            }
            _ => false,
        }
    }

    fn has_contextual_type_with_no_generic_types(&mut self, node: &Arc<Node>) -> bool {
        if !matches!(node.kind, SyntaxKind::Identifier | SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression) {
            return false;
        }
        if self.narrowable_reference_query_stack.contains(&node.id()) {
            return false;
        }
        self.narrowable_reference_query_stack.push(node.id());
        let result = self.has_contextual_type_with_no_generic_types_inner(node);
        self.narrowable_reference_query_stack.pop();
        result
    }

    fn has_contextual_type_with_no_generic_types_inner(&mut self, node: &Arc<Node>) -> bool {
        if let Some(parent) = node.parent()
            && let tsox_frontend::ast::NodeData::VariableDeclaration(d) = &parent.data
            && d.initializer.as_ref().is_some_and(|init| Arc::ptr_eq(init, node))
            && matches!(
                d.name.kind,
                SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
            )
            && pattern_element_count(&d.name) > 0
        {
            return true;
        }
        let Some(contextual) = self.get_contextual_type(node, ContextFlags::None) else {
            return false;
        };
        !self
            .constituent_types(&contextual)
            .iter()
            .any(|c| c.flags.intersects(TYPE_FLAGS_INSTANTIABLE))
    }

    pub fn get_flow_type_of_reference(
        &mut self,
        reference: &Arc<Node>,
        declared: &Arc<Type>,
    ) -> Arc<Type> {
        let Some(flow) = self
            .program
            .symbol_map()
            .flow_node_of(reference)
            .map(Arc::clone)
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

        if let Some(parent) = reference.parent() {
            if parent.kind == SyntaxKind::NonNullExpression
                && !narrowed.flags.contains(TypeFlags::Never)
                && self.type_is_never_after_removing_nullable(&narrowed)
            {
                return Arc::clone(declared);
            }
        }
        narrowed
    }

    pub(crate) fn type_is_never_after_removing_nullable(&self, t: &Arc<Type>) -> bool {
        if !self.strict_null_checks {
            return false;
        }
        if t.is_union() {
            return self
                .constituent_types(t)
                .iter()
                .all(|c| c.flags.intersects(TypeFlags::Null | TypeFlags::Undefined));
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

    pub(crate) fn type_contains_undefined_local(&self, t: &Arc<Type>) -> bool {
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

    pub(crate) fn flow_cache_key(
        &self,
        target: &FlowRef,
        flow: &Arc<FlowNode>,
        initial: &Arc<Type>,
    ) -> u64 {
        let ref_part = match target {
            FlowRef::Symbol(symbol) => symbol.id(),
            FlowRef::Node(node) => node.id(),
        };
        let flow_ptr = Arc::as_ptr(flow) as *const FlowNode as u64;
        let initial_ptr = initial.id as u64;

        (ref_part.rotate_left(17) ^ flow_ptr).rotate_left(29) ^ initial_ptr
    }

    pub(crate) fn type_at_flow_node(
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

    pub(crate) fn report_flow_control_error(&mut self, target: &FlowRef) {
        use tsox_frontend::ast::SyntaxKind;
        let Some(anchor) = target.anchor_node() else {
            return;
        };
        let mut block: Option<Arc<Node>> = None;
        let mut cur = anchor.parent();
        while let Some(n) = cur {
            let is_function_or_module_block = match n.kind {
                SyntaxKind::SourceFile | SyntaxKind::ModuleBlock => true,
                SyntaxKind::Block => n
                    .parent()
                    .as_ref()
                    .is_some_and(|p| tsox_frontend::ast::utilities::is_function_like_kind(p.kind)),
                _ => false,
            };
            if is_function_or_module_block {
                block = Some(Arc::clone(&n));
                break;
            }
            cur = n.parent();
        }
        let Some(block) = block else { return };

        let mut loc = block.loc;
        if let Some(stmts) = match &block.data {
            tsox_frontend::ast::NodeData::SourceFile(d) => Some(&d.statements),
            tsox_frontend::ast::NodeData::ModuleBlock(d) => Some(&d.statements),
            tsox_frontend::ast::NodeData::Block(d) => Some(&d.statements),
            _ => None,
        } && let Some(first) = stmts.nodes.first()
        {
            loc = tsox_core::core::text::TextRange::new(
                first.loc.pos as usize,
                first.loc.pos as usize + 1,
            );
        }
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            loc,
            tsox_core::diagnostics::messages_generated::
                THE_CONTAINING_FUNCTION_OR_MODULE_BODY_IS_TOO_LARGE_FOR_CONTROL_FLOW_ANALYSIS,
            Vec::new(),
        ));
    }
}

fn pattern_element_count(pattern: &Arc<Node>) -> usize {
    match &pattern.data {
        tsox_frontend::ast::NodeData::BindingPattern(d) => d.elements.len(),
        _ => 0,
    }
}
