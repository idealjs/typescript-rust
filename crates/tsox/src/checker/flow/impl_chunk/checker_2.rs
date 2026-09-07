#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn compute_type_at_flow_node(
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
            let antecedent_type =
                self.antecedent_type_at(declared, initial, flow, target, depth, query);
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
                return self.narrow_by_assertion_call(&antecedent_type, call_expr, target);
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
                let t = self.type_at_flow_node(declared, initial, ant, target, depth + 1, query);
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

    pub(crate) fn antecedent_type_at(
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

    pub(crate) fn constituent_types(&self, type_: &Arc<Type>) -> Vec<Arc<Type>> {
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
