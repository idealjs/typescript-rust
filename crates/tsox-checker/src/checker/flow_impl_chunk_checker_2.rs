#![allow(unused_imports)]

use crate::checker::checker_checker_checker::{FlowLoopInfo, FlowLoopKey};
use crate::checker::flow_impl_chunk::*;
use crate::checker::types_cached_type_kind::CacheHashKey;

impl Checker {
    pub(crate) fn compute_type_at_flow_node(
        &mut self,
        declared: &Arc<Type>,
        initial: &Arc<Type>,
        flow: &Arc<FlowNode>,
        target: &FlowRef,
        depth: u32,
        query: &mut FlowQuery,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("compute_type_at_flow_node"); 
        if flow.flags.contains(FlowFlags::UNREACHABLE) {
            return self.convert_auto_to_any(declared);
        }

        if flow.flags.contains(FlowFlags::START) {
            if let Some(container) = flow.node.clone() {
                let same_container = query
                    .flow_container
                    .as_ref()
                    .is_some_and(|fc| Arc::ptr_eq(fc, &container));
                let guards_pass = query.reference.as_ref().is_none_or(|reference| {
                    !tsox_frontend::ast::is_property_access_expression(reference)
                        && !tsox_frontend::ast::is_element_access_expression(reference)
                        && !(reference.kind == SyntaxKind::ThisKeyword
                            && !tsox_frontend::ast::is_arrow_function(&container))
                });
                if !same_container
                    && guards_pass
                    && let Some(creation_flow) = self
                        .program
                        .symbol_map()
                        .flow_node_of(&container)
                        .map(Arc::clone)
                {
                    return self.type_at_flow_node(
                        declared,
                        initial,
                        &creation_flow,
                        target,
                        depth,
                        query,
                    );
                }
            }
            return Arc::clone(initial);
        }

        if flow.flags.intersects(FlowFlags::CONDITION) {
            let kind = if flow.flags.contains(FlowFlags::TRUE_CONDITION) {
                NarrowKind::TrueBranch
            } else {
                NarrowKind::FalseBranch
            };
            let antecedent_type =
                self.antecedent_type_at(declared, initial, flow, target, depth, query);
            if let Some(expr) = &flow.node {
                return self.narrow_by_expression(&antecedent_type, expr, target, kind, depth);
            }
            return antecedent_type;
        }

        if flow.flags.contains(FlowFlags::ASSIGNMENT) {
            if let Some(expr) = &flow.node {
                if self.assignment_widens_target(expr, target) {
                    let ante = self.antecedent_type_at(declared, initial, flow, target, depth, query);
                    return self.get_base_type_of_literal_type(&ante);
                }

                if let Some(t) = self.assignment_flow_type(expr, target, declared, flow) {
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
                            let ante = self
                                .antecedent_type_at(declared, initial, flow, target, depth, query);
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
                    let pre_type =
                        self.antecedent_type_at(declared, initial, flow, target, depth, query);
                    if let Some(evolved) = self.evolve_array_at_mutation(node, &pre_type, target) {
                        return evolved;
                    }
                    return pre_type;
                }
            }

            return self.antecedent_type_at(declared, initial, flow, target, depth, query);
        }

        if flow.flags.contains(FlowFlags::CALL) {
            let antecedent_type =
                self.antecedent_type_at(declared, initial, flow, target, depth, query);
            if let Some(call_expr) = &flow.node {
                return self.narrow_by_assertion_call(&antecedent_type, declared, call_expr, target);
            }
            return antecedent_type;
        }

        if flow.flags.contains(FlowFlags::REDUCE_LABEL) {
            if let Some(reduce_target) = &flow.reduce_target {
                query
                    .reduce_labels
                    .push((Arc::clone(reduce_target), flow.antecedents.clone()));
                let t = self.antecedent_type_at(declared, initial, flow, target, depth, query);
                query.reduce_labels.pop();
                return t;
            }
            return self.antecedent_type_at(declared, initial, flow, target, depth, query);
        }

        if flow.flags.contains(FlowFlags::LOOP_LABEL) && flow.antecedents.len() > 1 {
            let ref_id = match target {
                FlowRef::Symbol(s) => s.id() as u64,
                FlowRef::Node(n) => n.id() as u64,
            };
            let key = FlowLoopKey {
                flow_node: Arc::clone(flow),
                ref_key: CacheHashKey { hi: ref_id, lo: declared.id as u64 },
            };
            if let Some(cached) = self.flow_loop_cache.get(&key) {
                return Arc::clone(cached);
            }
            if let Some(top) = self
                .flow_loop_stack
                .iter()
                .rev()
                .find(|info| info.key == key && !info.types.is_empty())
            {
                self.flow_loop_stack_hits += 1;
                let types = top.types.clone();
                return self.get_union_type(types);
            }
            let hits_before_walk = self.flow_loop_stack_hits;
            let mut ant_types: Vec<Arc<Type>> = Vec::new();
            for (idx, ant) in flow.antecedents.iter().enumerate() {
                let t;
                if idx == 0 {
                    t = self.type_at_flow_node(declared, initial, ant, target, depth + 1, query);
                } else {
                    self.flow_loop_stack
                        .push(FlowLoopInfo { key: key.clone(), types: ant_types.clone() });
                    let save_flow_type_cache = std::mem::take(&mut self.flow_type_cache);
                    let save_isolation = self.flow_back_edge_isolation;
                    let save_hits = self.flow_loop_stack_hits;
                    self.flow_back_edge_isolation = true;
                    t = self.type_at_flow_node(declared, initial, ant, target, depth + 1, query);
                    self.flow_back_edge_isolation = save_isolation;
                    self.flow_loop_stack_hits = save_hits;
                    self.flow_type_cache = save_flow_type_cache;
                    self.flow_loop_stack.pop();
                    if let Some(cached) = self.flow_loop_cache.get(&key) {
                        return Arc::clone(cached);
                    }
                }
                if !ant_types.iter().any(|u| Arc::ptr_eq(u, &t)) {
                    ant_types.push(Arc::clone(&t));
                }
                if Arc::ptr_eq(&t, declared) {
                    break;
                }
            }
            if ant_types.is_empty() {
                return Arc::clone(initial);
            }
            let result = self.flow_union_or_declared_type(declared, ant_types);
            if self.flow_loop_stack_hits == hits_before_walk {
                self.flow_loop_cache.insert(key, Arc::clone(&result));
            }
            return result;
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
                if Arc::ptr_eq(&t, declared) && Arc::ptr_eq(declared, initial) {
                    return t;
                }
                if !antecedent_types.iter().any(|u| Arc::ptr_eq(u, &t)) {
                    antecedent_types.push(t);
                }
            }

            if antecedent_types.len() == 1 {
                return self.flow_union_or_declared_type(
                    declared,
                    antecedent_types,
                );
            }

            if antecedent_types.is_empty() {
                return Arc::clone(declared);
            }
            return self.flow_union_or_declared_type(declared, antecedent_types);
        }

        self.antecedent_type_at(declared, initial, flow, target, depth, query)
    }

    /// Go getUnionOrEvolvingArrayType 的非 evolving 臂（flow.go:1369-1371）：
    /// 合并并集与声明型同为联合且成员表逐位相同时回替声明型（保留别名呈现，
    /// 如分支汇合重新拼出 A|B|C 时按声明型 MyUnion 显示）
    fn flow_union_or_declared_type(
        &mut self,
        declared: &Arc<Type>,
        types: Vec<Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("flow_union_or_declared_type");
        let result = self.get_union_type(types);
        if !Arc::ptr_eq(&result, declared)
            && result.flags.contains(TypeFlags::Union)
            && declared.flags.contains(TypeFlags::Union)
            && result
                .types()
                .is_some_and(|ts| declared.types().is_some_and(|ds| {
                    ts.len() == ds.len()
                        && ts.iter().zip(ds.iter()).all(|(t, d)| Arc::ptr_eq(t, d))
                }))
        {
            return Arc::clone(declared);
        }
        result
    }

    pub(crate) fn antecedent_type_at(
        &mut self,
        declared: &Arc<Type>,
        initial: &Arc<Type>,
        flow: &Arc<FlowNode>,
        target: &FlowRef,
        depth: u32,
        query: &mut FlowQuery,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("antecedent_type_at"); 
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

    pub(crate) fn constituent_types(&self, type_: &Arc<Type>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("constituent_types"); 
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
    pub(crate) fn convert_auto_to_any(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("convert_auto_to_any"); 
        if Arc::ptr_eq(t, &self.auto_type()) {
            return self.get_any_type();
        }
        if let Some(auto_arr) = self.auto_array_type.get()
            && Arc::ptr_eq(t, auto_arr)
        {
            return self.create_array_type(self.get_any_type());
        }
        Arc::clone(t)
    }
}
