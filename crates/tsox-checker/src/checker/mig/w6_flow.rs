#![allow(unused_imports)]
#![allow(dead_code)]

use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{
    is_binary_expression, is_binding_element, is_call_expression, is_case_clause, is_element_access_expression,
    is_for_in_statement, is_for_of_statement, is_function_expression_or_arrow_function, is_identifier, is_in_js_file,
    is_object_literal_method, is_parameter_declaration, is_parenthesized_expression, is_private_identifier,
    is_type_of_expression, is_variable_declaration, skip_parentheses, Node, NodeData, Symbol, SyntaxKind,
};
use tsox_frontend::ast::{FlowFlags, FlowNode};
use super::m2c::r18k3_defs::get_string_literal_value;
use crate::checker::mig::m2a::r18k8_flags::TYPE_FLAGS_UNION;
use super::m2e::r19k3_defs::non_dotted_name_cache_key;
use super::m2d_3::{is_evolving_array_type_list, FlowState, FlowType};
use super::m2c_3::some_type;
use super::wc1b::every_type;
use super::wc3::NodeAccessExt;
use crate::checker::utilities_token_is_identifier_or_keyword::{
    get_property_name_from_type, is_type_usable_as_property_name, is_unit_type,
};
use tsox_frontend::ast::mig::x1a::arguments;
use super::m2e::r19k3_defs::NodeAccessExtR19k3;
use crate::checker::flow_flow_max_depth::FlowRef;

impl Clone for FlowLoopKey {
    fn clone(&self) -> Self { ::tsox_core::fntrace::enter("clone"); 
        FlowLoopKey { flow_node: Arc::clone(&self.flow_node), ref_key: self.ref_key }
    }
}

thread_local! {
    static ANTECEDENT_TYPES: std::cell::RefCell<Vec<Arc<Type>>> = const { std::cell::RefCell::new(Vec::new()) };
    static SHARED_FLOWS: std::cell::RefCell<Vec<(usize, Option<Arc<Type>>, bool)>> = const { std::cell::RefCell::new(Vec::new()) };
}

impl Checker {
    pub fn get_type_at_flow_assignment(&mut self, f: &mut FlowState, flow: &Arc<FlowNode>) -> FlowType { ::tsox_core::fntrace::enter("get_type_at_flow_assignment"); 
        let node = flow.node.as_ref().unwrap();
        if self.is_matching_reference(f.reference.as_ref().unwrap(), node) {
            if !self.is_reachable_flow_node(flow) {
                return FlowType { t: Some(Arc::clone(&self.unreachable_never_type)), incomplete: false };
            }
            if get_assignment_target_kind(node) == AssignmentKind::Compound {
                let flow_type = self.get_type_at_flow_node(f, &flow.antecedent.clone().unwrap());
                let t = self.get_base_type_of_literal_type(flow_type.t.as_ref().unwrap());
                return self.new_flow_type(&t, flow_type.incomplete);
            }
            let declared_is_auto = f.declared_type.as_ref().is_some_and(|d| {
                self.auto_type.get().is_some_and(|a| Arc::ptr_eq(d, a))
                    || self.auto_array_type.get().is_some_and(|a| Arc::ptr_eq(d, a))
            });
            if declared_is_auto {
                if self.is_empty_array_assignment(node) {
                    return FlowType { t: Some(self.get_evolving_array_type(self.never_type.get().unwrap().clone())), incomplete: false };
                }
                let initial_or_assigned = self.get_initial_or_assigned_type(f, flow);
                let assigned_type = self.get_widened_literal_type(&initial_or_assigned);
                if self.is_type_assignable_to(&assigned_type, f.declared_type.as_ref().unwrap()) {
                    return FlowType { t: Some(assigned_type), incomplete: false };
                }
                return FlowType { t: Some(self.any_array_type()), incomplete: false };
            }
            let mut t = Arc::clone(f.declared_type.as_ref().unwrap());
            if is_in_compound_like_assignment(node) {
                t = self.get_base_type_of_literal_type(&t);
            }
            if t.flags.intersects(TypeFlags::Union) {
                let assigned = self.get_initial_or_assigned_type(f, flow);
                return FlowType {
                    t: Some(self.get_assignment_reduced_type(&t, &assigned)),
                    incomplete: false,
                };
            }
            return FlowType { t: Some(t), incomplete: false };
        }
        if self.contains_matching_reference(f.reference.as_ref().unwrap(), node) {
            if !self.is_reachable_flow_node(flow) {
                return FlowType { t: Some(Arc::clone(&self.unreachable_never_type)), incomplete: false };
            }
            if is_variable_declaration(node) && (is_in_js_file(node) || self.is_var_const_like(node)) {
                if let Some(init) = node.initializer()
                    && is_function_expression_or_arrow_function(init)
                {
                    return self.get_type_at_flow_node(f, &flow.antecedent.clone().unwrap());
                }
            }
            return FlowType { t: f.declared_type.clone(), incomplete: false };
        }
        if is_variable_declaration(node)
            && is_for_in_statement(&node.parent().unwrap())
            && (self.is_matching_reference(
                f.reference.as_ref().unwrap(),
                &node.parent().unwrap().parent().unwrap().expression().unwrap().clone(),
            ) || self.optional_chain_contains_reference(
                &node.parent().unwrap().parent().unwrap().expression().unwrap().clone(),
                f.reference.as_ref().unwrap(),
            ))
        {
            let flow_t = self
                .get_type_at_flow_node(f, &flow.antecedent.clone().unwrap())
                .t
                .unwrap();
            let finalized = self.finalize_evolving_array_type(&flow_t);
            return FlowType {
                t: Some(self.get_non_nullable_type_if_needed(&finalized)),
                incomplete: false,
            };
        }
        FlowType { t: None, incomplete: false }
    }

    pub fn get_initial_or_assigned_type(&mut self, f: &FlowState, flow: &Arc<FlowNode>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_initial_or_assigned_type"); 
        let node = flow.node.as_ref().unwrap();
        if is_variable_declaration(node) || is_binding_element(node) {
            let initial = self.get_initial_type(node);
            return self.get_narrowable_type_for_reference(&initial, f.reference.as_ref().unwrap());
        }
        let assigned = self.get_assigned_type(node);
        self.get_narrowable_type_for_reference(&assigned, f.reference.as_ref().unwrap())
    }

    pub fn get_type_at_flow_call(&mut self, f: &mut FlowState, flow: &Arc<FlowNode>) -> FlowType { ::tsox_core::fntrace::enter("get_type_at_flow_call"); 
        let node = flow.node.as_ref().unwrap();
        let signature = self.get_effects_signature(node);
        if let Some(signature) = signature {
            let predicate = self.get_type_predicate_of_signature(&signature);
            if let Some(predicate) = predicate
                && (predicate.kind == TypePredicateKind::AssertsThis || predicate.kind == TypePredicateKind::AssertsIdentifier)
            {
                let flow_type = self.get_type_at_flow_node(f, &flow.antecedent.clone().unwrap());
                let t = self.finalize_evolving_array_type(flow_type.t.as_ref().unwrap());
                let narrowed_type = if predicate.t.is_some() {
                    self.narrow_type_by_type_predicate(f, &t, &predicate, node, true)
                } else if predicate.kind == TypePredicateKind::AssertsIdentifier
                    && predicate.parameter_index >= 0
                    && (predicate.parameter_index as usize) < arguments(node).len()
                {
                    self.narrow_type_by_assertion(f, &t, &arguments(node)[predicate.parameter_index as usize])
                } else {
                    Arc::clone(&t)
                };
                if Arc::ptr_eq(&narrowed_type, &t) {
                    return flow_type;
                }
                return self.new_flow_type(&narrowed_type, flow_type.incomplete);
            }
            if self
                .get_return_type_of_signature(&signature)
                .is_some_and(|rt| rt.flags.intersects(TypeFlags::Never))
            {
                return FlowType { t: Some(Arc::clone(&self.unreachable_never_type)), incomplete: false };
            }
        }
        FlowType { t: None, incomplete: false }
    }

    pub fn get_type_at_flow_condition(&mut self, f: &mut FlowState, flow: &Arc<FlowNode>) -> FlowType { ::tsox_core::fntrace::enter("get_type_at_flow_condition"); 
        let flow_type = self.get_type_at_flow_node(f, &flow.antecedent.clone().unwrap());
        if flow_type.t.as_ref().unwrap().flags.intersects(TypeFlags::Never) {
            return flow_type;
        }
        let assume_true = flow.flags.contains(FlowFlags::TRUE_CONDITION);
        let non_evolving_type = self.finalize_evolving_array_type(flow_type.t.as_ref().unwrap());
        let narrowed_type = self.narrow_type(f, &non_evolving_type, flow.node.as_ref().unwrap(), assume_true);
        if Arc::ptr_eq(&narrowed_type, &non_evolving_type) {
            return flow_type;
        }
        self.new_flow_type(&narrowed_type, flow_type.incomplete)
    }

    pub fn get_type_at_switch_clause(&mut self, f: &mut FlowState, flow: &Arc<FlowNode>) -> FlowType { ::tsox_core::fntrace::enter("get_type_at_switch_clause"); 
        let data_node = flow.node.as_ref().unwrap();
        let switch_statement = flow.switch_statement.as_ref().unwrap();
        let expr = skip_parentheses(switch_statement.expression().unwrap());
        let flow_type = self.get_type_at_flow_node(f, &flow.antecedent.clone().unwrap());
        let mut t = Arc::clone(flow_type.t.as_ref().unwrap());
        if self.is_matching_reference(f.reference.as_ref().unwrap(), &expr) {
            t = self.narrow_type_by_switch_on_discriminant(&t, data_node);
        } else if expr.kind == SyntaxKind::TypeOfExpression
            && self.is_matching_reference(f.reference.as_ref().unwrap(), expr.expression().unwrap())
        {
            t = self.narrow_type_by_switch_on_type_of(&t, data_node);
        } else if expr.kind == SyntaxKind::TrueKeyword {
            t = self.narrow_type_by_switch_on_true(f, &t, data_node);
        } else if self.strict_null_checks {
            if self.optional_chain_contains_reference(&expr, f.reference.as_ref().unwrap()) {
                t = self.narrow_type_by_switch_optional_chain_containment(&t, data_node, &|t: &Arc<Type>| {
                    !t.flags.intersects(TypeFlags::Undefined | TypeFlags::Never)
                });
            } else if is_type_of_expression(&expr)
                && self.optional_chain_contains_reference(expr.expression().unwrap(), f.reference.as_ref().unwrap())
            {
                t = self.narrow_type_by_switch_optional_chain_containment(&t, data_node, &|t: &Arc<Type>| {
                    !(t.flags.intersects(TypeFlags::Never)
                        || t.flags.intersects(TypeFlags::StringLiteral) && get_string_literal_value(t) == "undefined")
                });
            }
        }
        if Arc::ptr_eq(&t, flow_type.t.as_ref().unwrap()) {
            return flow_type;
        }
        self.new_flow_type(&t, flow_type.incomplete)
    }

    pub fn get_type_at_flow_branch_label(
        &mut self,
        f: &mut FlowState,
        flow: &Arc<FlowNode>,
        antecedents: &[Arc<FlowNode>],
    ) -> FlowType { ::tsox_core::fntrace::enter("get_type_at_flow_branch_label"); 
        let antecedent_start = ANTECEDENT_TYPES.with_borrow(Vec::len);
        let mut subtype_reduction = false;
        let mut seen_incomplete = false;
        let mut bypass_flow: Option<Arc<FlowNode>> = None;
        for antecedent in antecedents {
            if bypass_flow.is_none()
                && antecedent.flags.contains(FlowFlags::SWITCH_CLAUSE)
                && matches!(antecedent.clause_range, Some((start, end)) if start == end)
            {
                bypass_flow = Some(Arc::clone(antecedent));
                continue;
            }
            let flow_type = self.get_type_at_flow_node(f, antecedent);
            let flow_t = flow_type.t.as_ref().unwrap();
            if f.declared_type.as_ref() == Some(flow_t) && f.declared_type == f.initial_type {
                ANTECEDENT_TYPES.with_borrow_mut(|a| a.truncate(antecedent_start));
                return FlowType { t: Some(Arc::clone(flow_t)), incomplete: false };
            }
            if !ANTECEDENT_TYPES.with_borrow(|a| a[antecedent_start..].contains(flow_t)) {
                ANTECEDENT_TYPES.with_borrow_mut(|a| a.push(Arc::clone(flow_t)));
            }
            if !self.is_type_subset_of(flow_t, f.initial_type.as_ref().unwrap()) {
                subtype_reduction = true;
            }
            if flow_type.incomplete {
                seen_incomplete = true;
            }
        }
        if let Some(bypass_flow) = bypass_flow {
            let flow_type = self.get_type_at_flow_node(f, &bypass_flow);
            let flow_t = flow_type.t.as_ref().unwrap();
            if !flow_t.flags.intersects(TypeFlags::Never)
                && !ANTECEDENT_TYPES.with_borrow(|a| a[antecedent_start..].contains(flow_t))
                && !self.is_exhaustive_switch_statement(bypass_flow.switch_statement.as_ref().unwrap())
            {
                if f.declared_type.as_ref() == Some(flow_t) && f.declared_type == f.initial_type {
                    ANTECEDENT_TYPES.with_borrow_mut(|a| a.truncate(antecedent_start));
                    return FlowType { t: Some(Arc::clone(flow_t)), incomplete: false };
                }
                ANTECEDENT_TYPES.with_borrow_mut(|a| a.push(Arc::clone(flow_t)));
                if !self.is_type_subset_of(flow_t, f.initial_type.as_ref().unwrap()) {
                    subtype_reduction = true;
                }
                if flow_type.incomplete {
                    seen_incomplete = true;
                }
            }
        }
        let subtype_reduction = if subtype_reduction { UnionReduction::Subtype } else { UnionReduction::Literal };
        let union_input = ANTECEDENT_TYPES.with_borrow(|a| a[antecedent_start..].to_vec());
        let union_type = self.get_union_or_evolving_array_type(f, &union_input, subtype_reduction);
        let result = self.new_flow_type(&union_type, seen_incomplete);
        ANTECEDENT_TYPES.with_borrow_mut(|a| a.truncate(antecedent_start));
        result
    }

    pub fn get_union_or_evolving_array_type(
        &mut self,
        f: &FlowState,
        types: &[Arc<Type>],
        subtype_reduction: UnionReduction,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_union_or_evolving_array_type"); 
        if is_evolving_array_type_list(types) {
            let element_types = types.iter().map(|t| self.get_element_type_of_evolving_array_type(t)).collect::<Vec<_>>();
            let union_type = self.get_union_type(element_types);
            return self.get_evolving_array_type(union_type);
        }
        let finalized = types.iter().map(|t| self.finalize_evolving_array_type(t)).collect::<Vec<_>>();
        let union = self.get_union_type_ex(finalized, subtype_reduction);
        let result = self.recombine_unknown_type(&union);
        if f.declared_type.as_ref() != Some(&result)
            && result.flags.intersects(TypeFlags::Union)
            && f.declared_type.as_ref().is_some_and(|d| d.flags.intersects(TypeFlags::Union))
        {
            if let (TypeData::Union(r), TypeData::Union(d)) = (&result.data, &f.declared_type.as_ref().unwrap().data) {
                if r.union_or_intersection.types == d.union_or_intersection.types {
                    return Arc::clone(f.declared_type.as_ref().unwrap());
                }
            }
        }
        result
    }

    pub fn get_type_at_flow_loop_label(&mut self, f: &mut FlowState, flow: &Arc<FlowNode>) -> FlowType { ::tsox_core::fntrace::enter("get_type_at_flow_loop_label"); 
        if f.ref_key.is_zero() {
            f.ref_key = self.get_flow_reference_key(f);
        }
        if f.ref_key == non_dotted_name_cache_key() {
            return FlowType { t: f.declared_type.clone(), incomplete: false };
        }
        let key = FlowLoopKey { flow_node: Arc::clone(flow), ref_key: f.ref_key.clone() };
        if let Some(cached) = self.flow_loop_cache.get(&key) {
            return FlowType { t: Some(Arc::clone(cached)), incomplete: false };
        }
        let mut loop_types: Option<Vec<Arc<Type>>> = None;
        for loop_info in &self.flow_loop_stack {
            if loop_info.key == key && !loop_info.types.is_empty() {
                loop_types = Some(loop_info.types.clone());
                break;
            }
        }
        if let Some(types) = loop_types {
            let union_type = self.get_union_or_evolving_array_type(f, &types, UnionReduction::Literal);
            return self.new_flow_type(&union_type, true);
        }
        let mut antecedent_types: Vec<Arc<Type>> = Vec::with_capacity(4);
        let mut subtype_reduction = false;
        let mut first_antecedent_type = FlowType { t: None, incomplete: false };
        for antecedent in &flow.antecedents {
            let flow_type;
            if first_antecedent_type.t.is_none() {
                first_antecedent_type = self.get_type_at_flow_node(f, antecedent);
                flow_type = FlowType { t: first_antecedent_type.t.clone(), incomplete: first_antecedent_type.incomplete };
            } else {
                self.flow_loop_stack.push(FlowLoopInfo { key: key.clone(), types: antecedent_types.clone() });
                let save_flow_type_cache = std::mem::take(&mut self.flow_type_cache);
                flow_type = self.get_type_at_flow_node(f, antecedent);
                self.flow_type_cache = save_flow_type_cache;
                self.flow_loop_stack.pop();
                if let Some(cached) = self.flow_loop_cache.get(&key) {
                    return FlowType { t: Some(Arc::clone(cached)), incomplete: false };
                }
            }
            let flow_t = flow_type.t.as_ref().unwrap();
            if !antecedent_types.contains(flow_t) {
                antecedent_types.push(Arc::clone(flow_t));
            }
            if !self.is_type_subset_of(flow_t, f.initial_type.as_ref().unwrap()) {
                subtype_reduction = true;
            }
            if f.declared_type.as_ref() == Some(flow_t) {
                break;
            }
        }
        let subtype_reduction = if subtype_reduction { UnionReduction::Subtype } else { UnionReduction::Literal };
        let result = self.get_union_or_evolving_array_type(f, &antecedent_types, subtype_reduction);
        if first_antecedent_type.incomplete {
            return self.new_flow_type(&result, true);
        }
        self.flow_loop_cache.insert(key, Arc::clone(&result));
        FlowType { t: Some(result), incomplete: false }
    }

    pub fn get_type_at_flow_array_mutation(&mut self, f: &mut FlowState, flow: &Arc<FlowNode>) -> FlowType { ::tsox_core::fntrace::enter("get_type_at_flow_array_mutation"); 
        let declared_is_auto = f.declared_type.as_ref().is_some_and(|d| {
            self.auto_type.get().is_some_and(|a| Arc::ptr_eq(d, a))
                || self.auto_array_type.get().is_some_and(|a| Arc::ptr_eq(d, a))
        });
        if declared_is_auto {
            let node = flow.node.as_ref().unwrap();
            let expr = if is_call_expression(node) {
                node.expression().unwrap().expression().unwrap().clone()
            } else {
                node.as_binary_expression().left.expression().unwrap().clone()
            };
            if self.is_matching_reference(f.reference.as_ref().unwrap(), &self.get_reference_candidate(&expr)) {
                let flow_type = self.get_type_at_flow_node(f, &flow.antecedent.clone().unwrap());
                let flow_t = flow_type.t.as_ref().unwrap();
                if flow_t.object_flags.intersects(ObjectFlags::EvolvingArray) {
                    let mut evolved_type = Arc::clone(flow_t);
                    if is_call_expression(node) {
                        for arg in arguments(node) {
                            let arg_type = self.get_context_free_type_of_expression(arg);
                            evolved_type = self.add_evolving_array_element_type(&evolved_type, arg_type);
                        }
                    } else {
                        let index_type = self.get_context_free_type_of_expression(
                            &node.as_binary_expression().left.as_element_access_expression().argument_expression,
                        );
                        if self.is_type_assignable_to_kind(&index_type, TYPE_FLAGS_NUMBER_LIKE) {
                            let right_type = self.get_context_free_type_of_expression(&node.as_binary_expression().right);
                            evolved_type = self.add_evolving_array_element_type(&evolved_type, right_type);
                        }
                    }
                    return self.new_flow_type(&evolved_type, flow_type.incomplete);
                }
                return flow_type;
            }
        }
        FlowType { t: None, incomplete: false }
    }

    pub fn get_type_of_switch_clause(&mut self, clause: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_switch_clause"); 
        if clause.kind == SyntaxKind::CaseClause {
            let expr_type = self.check_expression_ex(clause.expression().unwrap(), CheckMode::Normal);
            return self.get_regular_type_of_literal_type(&expr_type);
        }
        Arc::clone(self.never_type.get().unwrap())
    }

    pub fn get_type_of_dotted_name(&mut self, node: &Arc<Node>, diagnostic: Option<&Arc<Node>>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_of_dotted_name"); 
        if !node.flags.contains(tsox_frontend::ast::NodeFlags::InWithStatement) {
            match node.kind {
                SyntaxKind::Identifier => {
                    let symbol = self.get_resolved_symbol(node)?;
                    let symbol = self.get_export_symbol_of_value_symbol_if_exported(&symbol);
                    return self.get_explicit_type_of_symbol(&symbol, None);
                }
                SyntaxKind::ThisKeyword => {
                    return self.get_explicit_this_type(node);
                }
                SyntaxKind::SuperKeyword => {
                    self.check_super_expression(node);
                    return Some(self.get_type_of_node(node));
                }
                SyntaxKind::PropertyAccessExpression => {
                    let t = self.get_type_of_dotted_name(&node.expression().unwrap().clone(), diagnostic)?;
                    let name = node.name().unwrap().clone();
                    let prop = if is_private_identifier(&name) {
                        match &t.symbol {
                            Some(s) => self.get_property_of_type(
                                &t,
                                &crate::binder::mig::m3h::get_symbol_name_for_private_identifier(s, &name.text()),
                            ),
                            None => None,
                        }
                    } else {
                        self.get_property_of_type(&t, &name.text())
                    };
                    if let Some(prop) = prop {
                        return self.get_explicit_type_of_symbol(&prop, None);
                    }
                    return None;
                }
                SyntaxKind::ParenthesizedExpression => {
                    return self.get_type_of_dotted_name(&node.expression().unwrap().clone(), diagnostic);
                }
                _ => {}
            }
        }
        None
    }

    pub fn get_type_of_destructured_array_element(&mut self, t: &Arc<Type>, index: usize) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_destructured_array_element"); 
        if every_type_is_tuple_like_w6(self, t) {
            if let Some(element_type) = self.get_tuple_element_type(t, index) {
                return element_type;
            }
        }
        let element_type = self.check_iterated_type_or_element_type(IterationUse::Destructuring, t, None);
        self.include_undefined_in_index_signature(Some(&element_type)).unwrap()
    }

    pub fn get_type_of_destructured_spread_expression(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_destructured_spread_expression"); 
        let element_type = self.check_iterated_type_or_element_type(IterationUse::Destructuring, t, None);
        self.create_array_type(element_type)
    }

    pub fn get_type_of_destructured_property(&mut self, t: &Arc<Type>, name: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_destructured_property"); 
        let Some(name_type) = self.get_literal_type_from_property_name(name) else {
            return Arc::clone(self.error_type.get().unwrap());
        };
        if !is_type_usable_as_property_name(&name_type) {
            return Arc::clone(self.error_type.get().unwrap());
        }
        let text = get_property_name_from_type(&name_type);
        if let Some(prop_type) = self.get_type_of_property_of_type(t, &text) {
            return prop_type;
        }
        if let Some(index_info) = self.get_applicable_index_info_for_name(t, &text) {
            return self.include_undefined_in_index_signature(index_info.value_type.as_ref()).unwrap();
        }
        Arc::clone(self.error_type.get().unwrap())
    }

    pub fn is_destructuring_assignment_target(&self, parent: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_destructuring_assignment_target"); 
        let pp = parent.parent().unwrap();
        (is_binary_expression(&pp) && Arc::ptr_eq(&pp.as_binary_expression().left, parent))
            || (is_for_of_statement(&pp) && Arc::ptr_eq(pp.initializer().unwrap(), parent))
    }

    pub fn is_constructed_by(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_constructed_by"); 
        if (source.flags.intersects(TypeFlags::Object) && source.object_flags.intersects(ObjectFlags::Class))
            || (target.flags.intersects(TypeFlags::Object) && target.object_flags.intersects(ObjectFlags::Class))
        {
            return match (&source.symbol, &target.symbol) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            };
        }
        self.is_type_subtype_of(source, target)
    }
}

const TYPE_FLAGS_NUMBER_LIKE: TypeFlags = TypeFlags::Number.union(TypeFlags::NumberLiteral);
const TYPE_FLAGS_PRIMITIVE: TypeFlags = TypeFlags::String
    .union(TypeFlags::Number)
    .union(TypeFlags::BigInt)
    .union(TypeFlags::Boolean)
    .union(TypeFlags::ESSymbol)
    .union(TypeFlags::StringLiteral)
    .union(TypeFlags::NumberLiteral)
    .union(TypeFlags::BigIntLiteral)
    .union(TypeFlags::BooleanLiteral);

fn is_optional_chain_w6(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_optional_chain_w6"); 
    node.flags.contains(tsox_frontend::ast::NodeFlags::OptionalChain)
        && matches!(
            node.kind,
            SyntaxKind::PropertyAccessExpression
                | SyntaxKind::ElementAccessExpression
                | SyntaxKind::CallExpression
                | SyntaxKind::NonNullExpression
        )
}

fn union_member_ptr(u: &Arc<Type>, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("union_member_ptr");     let TypeData::Union(data) = &u.data else {
        return false;
    };
    data.union_or_intersection.types.iter().any(|m| m.id == t.id)
}

fn union_contains_type_w6(checker: &Checker, u: &Arc<Type>, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("union_contains_type_w6"); 
    if union_member_ptr(u, t) {
        return true;
    }
    let primitive = if t.flags.contains(TypeFlags::StringLiteral) {
        Some(checker.string_type())
    } else if t.flags.intersects(TypeFlags::Enum | TypeFlags::NumberLiteral) {
        Some(checker.number_type())
    } else if t.flags.contains(TypeFlags::BigIntLiteral) {
        Some(checker.bigint_type())
    } else if t.flags.contains(TypeFlags::UniqueESSymbol) {
        Some(checker.es_symbol_type())
    } else {
        None
    };
    primitive.is_some_and(|p| union_member_ptr(u, &p))
}

pub fn get_branch_label_antecedents(
    flow: &Arc<FlowNode>,
    reduce_labels: &[Arc<tsox_frontend::ast::mig::m3e::FlowReduceLabelData>],
) -> Vec<Arc<FlowNode>> { ::tsox_core::fntrace::enter("get_branch_label_antecedents"); 
    for data in reduce_labels.iter().rev() {
        if Arc::ptr_eq(&data.target, flow) {
            if let Some(antecedents) = &data.antecedents {
                return antecedents.clone();
            }
        }
    }
    flow.antecedents.clone()
}

impl Checker {
    pub fn get_type_at_flow_node(&mut self, f: &mut FlowState, flow: &Arc<FlowNode>) -> FlowType { ::tsox_core::fntrace::enter("get_type_at_flow_node"); 
        let mut flow = flow;
        let mut owned_antecedent: Option<Arc<FlowNode>> = None;
        if f.depth == 2000 {
            self.flow_analysis_disabled = true;
            self.report_flow_control_error(&FlowRef::Node(Arc::clone(f.reference.as_ref().unwrap())));
            return FlowType { t: Some(Arc::clone(self.error_type.get().unwrap())), incomplete: false };
        }
        f.depth += 1;
        let mut shared_flow: Option<usize> = None;
        let result;
        loop {
            let flags = flow.flags;
            if flags.contains(FlowFlags::SHARED) {
                let key = Arc::as_ptr(flow) as usize;
                let start = f.shared_flow_start as usize;
                let cached = SHARED_FLOWS.with(|s| {
                    let s = s.borrow();
                    s.iter().skip(start).find(|e| e.0 == key).map(|e| (e.1.clone(), e.2))
                });
                if let Some((t, incomplete)) = cached {
                    f.depth -= 1;
                    return FlowType { t, incomplete };
                }
                shared_flow = Some(key);
            }
            let mut t;
            if flow.flags.contains(FlowFlags::ASSIGNMENT) {
                t = self.get_type_at_flow_assignment(f, flow);
                if t.t.is_none() {
                    flow = flow.antecedent.as_ref().unwrap();
                    continue;
                }
            } else if flow.flags.contains(FlowFlags::CALL) {
                t = self.get_type_at_flow_call(f, flow);
                if t.t.is_none() {
                    flow = flow.antecedent.as_ref().unwrap();
                    continue;
                }
            } else if flow.flags.contains(FlowFlags::CONDITION) {
                t = self.get_type_at_flow_condition(f, flow);
            } else if flow.flags.contains(FlowFlags::SWITCH_CLAUSE) {
                t = self.get_type_at_switch_clause(f, flow);
            } else if flow.flags.contains(FlowFlags::BRANCH_LABEL) {
                let antecedents = get_branch_label_antecedents(flow, &f.reduce_labels);
                if antecedents.len() == 1 {
                    owned_antecedent = Some(Arc::clone(&antecedents[0]));
                    flow = owned_antecedent.as_ref().unwrap();
                    continue;
                }
                t = self.get_type_at_flow_branch_label(f, flow, &antecedents);
            } else if flow.flags.contains(FlowFlags::LOOP_LABEL) {
                if flow.antecedents.len() == 1 {
                    flow = &flow.antecedents[0];
                    continue;
                }
                t = self.get_type_at_flow_loop_label(f, flow);
            } else if flow.flags.contains(FlowFlags::ARRAY_MUTATION) {
                t = self.get_type_at_flow_array_mutation(f, flow);
                if t.t.is_none() {
                    flow = flow.antecedent.as_ref().unwrap();
                    continue;
                }
            } else if flow.flags.contains(FlowFlags::REDUCE_LABEL) {
                let reduce_data = tsox_frontend::ast::mig::x1a::as_flow_reduce_label_data(flow.node.as_ref().unwrap());
                let data = Arc::new(tsox_frontend::ast::mig::m3e::FlowReduceLabelData {
                    target: Arc::clone(&reduce_data.target),
                    antecedents: reduce_data.antecedents.clone(),
                });
                f.reduce_labels.push(data);
                t = self.get_type_at_flow_node(f, flow.antecedent.as_ref().unwrap());
                f.reduce_labels.pop();
            } else if flow.flags.contains(FlowFlags::START) {
                let container = flow.node.clone();
                if let Some(container) = container {
                    let is_same_container = f.flow_container.as_ref().is_some_and(|fc| Arc::ptr_eq(fc, &container));
                    let reference_is_property_access = tsox_frontend::ast::is_property_access_expression(f.reference.as_ref().unwrap());
                    let reference_is_element_access = is_element_access_expression(f.reference.as_ref().unwrap());
                    let reference_is_this_arrow = f.reference.as_ref().unwrap().kind == SyntaxKind::ThisKeyword
                        && !tsox_frontend::ast::is_arrow_function(&container);
                    if !is_same_container && !reference_is_property_access && !reference_is_element_access && !reference_is_this_arrow {
                        if let Some(next_flow) = super::m2d_3::get_flow_node_of_node(&container) {
                            owned_antecedent = Some(next_flow);
                            flow = owned_antecedent.as_ref().unwrap();
                            continue;
                        }
                    }
                }
                t = FlowType { t: f.initial_type.clone(), incomplete: false };
            } else {
                t = FlowType { t: Some(self.convert_auto_to_any(f.declared_type.as_ref().unwrap())), incomplete: false };
            }
            if let Some(key) = shared_flow {
                SHARED_FLOWS.with(|s| s.borrow_mut().push((key, t.t.clone(), t.incomplete)));
            }
            result = t;
            break;
        }
        f.depth -= 1;
        result
    }

    pub fn narrow_type(&mut self, f: &mut FlowState, t: &Arc<Type>, expr: &Arc<Node>, assume_true: bool) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type"); 
        let parent_is_qq = match expr.parent() {
            Some(parent) if is_binary_expression(&parent) => {
                let binary = parent.as_binary_expression();
                (binary.operator_token.kind == SyntaxKind::QuestionQuestionToken
                    || binary.operator_token.kind == SyntaxKind::QuestionQuestionEqualsToken)
                    && Arc::ptr_eq(&binary.left, expr)
            }
            _ => false,
        };
        if tsox_frontend::ast::mig::w7a::is_expression_of_optional_chain_root(expr) || parent_is_qq {
            return self.narrow_type_by_optionality(f, t, expr, assume_true);
        }
        match expr.kind {
            SyntaxKind::Identifier => {
                if !self.is_matching_reference(f.reference.as_ref().unwrap(), expr) && self.inline_level < 5 {
                    if let Some(symbol) = self.get_resolved_symbol(expr) {
                        if self.is_constant_variable(&symbol) {
                            if let Some(declaration) = &symbol.value_declaration {
                                if is_variable_declaration(declaration)
                                    && declaration.type_().is_none()
                                    && declaration.initializer().is_some()
                                    && self.is_constant_reference(f.reference.as_ref().unwrap())
                                {
                                    self.inline_level += 1;
                                    let result = self.narrow_type(f, t, declaration.initializer().unwrap(), assume_true);
                                    self.inline_level -= 1;
                                    return result;
                                }
                            }
                        }
                    }
                }
                self.narrow_type_by_truthiness(f, t, expr, assume_true)
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword | SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                self.narrow_type_by_truthiness(f, t, expr, assume_true)
            }
            SyntaxKind::CallExpression => self.narrow_type_by_call_expression(f, t, expr, assume_true),
            SyntaxKind::ParenthesizedExpression | SyntaxKind::NonNullExpression | SyntaxKind::SatisfiesExpression => {
                self.narrow_type(f, t, expr.expression().unwrap(), assume_true)
            }
            SyntaxKind::BinaryExpression => self.narrow_type_by_binary_expression(f, t, expr.as_binary_expression(), assume_true),
            SyntaxKind::PrefixUnaryExpression => {
                let unary = expr.as_prefix_unary_expression();
                if unary.operator == SyntaxKind::ExclamationToken {
                    return self.narrow_type(f, t, &unary.operand, !assume_true);
                }
                Arc::clone(t)
            }
            _ => Arc::clone(t),
        }
    }

    pub fn narrow_type_by_optionality(&mut self, f: &mut FlowState, t: &Arc<Type>, expr: &Arc<Node>, assume_present: bool) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_optionality"); 
        if self.is_matching_reference(f.reference.as_ref().unwrap(), expr) {
            let facts = if assume_present { TypeFacts::NE_UNDEFINED_OR_NULL } else { TypeFacts::EQ_UNDEFINED_OR_NULL };
            return self.get_adjusted_type_with_facts(t, facts);
        }
        if let Some(access) = self.get_discriminant_property_access(f, expr, t) {
            let facts = if assume_present { TypeFacts::NE_UNDEFINED_OR_NULL } else { TypeFacts::EQ_UNDEFINED_OR_NULL };
            return self.narrow_type_by_discriminant(t, &access, &|checker, t| checker.get_type_with_facts(t, facts));
        }
        Arc::clone(t)
    }

    pub fn narrow_type_by_truthiness(&mut self, f: &mut FlowState, t: &Arc<Type>, expr: &Arc<Node>, assume_true: bool) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_truthiness"); 
        let mut t = Arc::clone(t);
        if self.is_matching_reference(f.reference.as_ref().unwrap(), expr) {
            let facts = if assume_true { TypeFacts::TRUTHY } else { TypeFacts::FALSY };
            return self.get_adjusted_type_with_facts(&t, facts);
        }
        if self.strict_null_checks && assume_true && self.optional_chain_contains_reference(expr, f.reference.as_ref().unwrap()) {
            t = self.get_adjusted_type_with_facts(&t, TypeFacts::NE_UNDEFINED_OR_NULL);
        }
        if let Some(access) = self.get_discriminant_property_access(f, expr, &t) {
            let facts = if assume_true { TypeFacts::TRUTHY } else { TypeFacts::FALSY };
            return self.narrow_type_by_discriminant(&t, &access, &|checker, t| checker.get_type_with_facts(t, facts));
        }
        t
    }

    pub fn narrow_type_by_call_expression(&mut self, f: &mut FlowState, t: &Arc<Type>, call_expression: &Arc<Node>, assume_true: bool) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_call_expression"); 
        if self.has_matching_argument(call_expression, f.reference.as_ref().unwrap()) {
            let predicate = if assume_true || !crate::checker::utilities_has_only_expression_initialization::is_call_chain(call_expression) {
                self.get_effects_signature(call_expression)
                    .and_then(|signature| self.get_type_predicate_of_signature(&signature).cloned())
            } else {
                None
            };
            if let Some(predicate) = predicate
                && (predicate.kind == TypePredicateKind::This || predicate.kind == TypePredicateKind::Identifier)
            {
                return self.narrow_type_by_type_predicate(f, t, &predicate, call_expression, assume_true);
            }
        }
        Arc::clone(t)
    }

    pub fn narrow_type_by_type_predicate(
        &mut self,
        f: &mut FlowState,
        t: &Arc<Type>,
        predicate: &TypePredicate,
        call_expression: &Arc<Node>,
        assume_true: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_type_predicate"); 
        if let Some(predicate_t) = &predicate.t {
            let is_global_object = self.global_object_type.get().is_some_and(|g| Arc::ptr_eq(predicate_t, g));
            let is_global_function = self.global_function_type.get().is_some_and(|g| Arc::ptr_eq(predicate_t, g));
            if !(t.flags.intersects(TypeFlags::Any) && (is_global_object || is_global_function)) {
                let predicate_argument = self.get_type_predicate_argument(predicate, call_expression);
                if let Some(predicate_argument) = predicate_argument {
                    if self.is_matching_reference(f.reference.as_ref().unwrap(), &predicate_argument) {
                        return self.get_narrowed_type(t, predicate_t, assume_true, false);
                    }
                    if self.strict_null_checks
                        && self.optional_chain_contains_reference(&predicate_argument, f.reference.as_ref().unwrap())
                        && (assume_true && !self.has_type_facts(predicate_t, TypeFacts::EQ_UNDEFINED)
                            || !assume_true && predicate_t.flags.intersects(TypeFlags::Undefined | TypeFlags::Null))
                    {
                        return self.get_adjusted_type_with_facts(t, TypeFacts::NE_UNDEFINED_OR_NULL);
                    }
                    if let Some(access) = self.get_discriminant_property_access(f, &predicate_argument, t) {
                        let predicate_t = Arc::clone(predicate_t);
                        return self.narrow_type_by_discriminant(t, &access, &move |checker, t| {
                            checker.get_narrowed_type(t, &predicate_t, assume_true, false)
                        });
                    }
                }
            }
        }
        Arc::clone(t)
    }

    pub fn narrow_type_by_assertion(&mut self, f: &mut FlowState, t: &Arc<Type>, expr: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_assertion"); 
        let node = skip_parentheses(expr);
        if node.kind == SyntaxKind::FalseKeyword {
            return Arc::clone(&self.unreachable_never_type);
        }
        if node.kind == SyntaxKind::BinaryExpression {
            let binary = node.as_binary_expression();
            if binary.operator_token.kind == SyntaxKind::AmpersandAmpersandToken {
                let left = Arc::clone(&binary.left);
                let right = Arc::clone(&binary.right);
                let inner = self.narrow_type_by_assertion(f, t, &left);
                return self.narrow_type_by_assertion(f, &inner, &right);
            }
            if binary.operator_token.kind == SyntaxKind::BarBarToken {
                let left = Arc::clone(&binary.left);
                let right = Arc::clone(&binary.right);
                let l = self.narrow_type_by_assertion(f, t, &left);
                let r = self.narrow_type_by_assertion(f, t, &right);
                return self.get_union_type(vec![l, r]);
            }
        }
        self.narrow_type(f, t, &node, true)
    }

    pub fn narrow_type_by_binary_expression(
        &mut self,
        f: &mut FlowState,
        t: &Arc<Type>,
        expr: &tsox_frontend::ast::node_data_generated::BinaryExpressionData,
        assume_true: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_binary_expression"); 
        match expr.operator_token.kind {
            SyntaxKind::EqualsToken | SyntaxKind::BarBarEqualsToken | SyntaxKind::AmpersandAmpersandEqualsToken | SyntaxKind::QuestionQuestionEqualsToken => {
                let inner = self.narrow_type(f, t, &expr.right, assume_true);
                self.narrow_type_by_truthiness(f, &inner, &expr.left, assume_true)
            }
            SyntaxKind::EqualsEqualsToken | SyntaxKind::ExclamationEqualsToken | SyntaxKind::EqualsEqualsEqualsToken | SyntaxKind::ExclamationEqualsEqualsToken => {
                let operator = expr.operator_token.kind;
                let left = self.get_reference_candidate(&expr.left);
                let right = self.get_reference_candidate(&expr.right);
                if left.kind == SyntaxKind::TypeOfExpression && tsox_frontend::ast::is_string_literal_like(&right) {
                    return self.narrow_type_by_typeof(f, t, &left, operator, &right, assume_true);
                }
                if right.kind == SyntaxKind::TypeOfExpression && tsox_frontend::ast::is_string_literal_like(&left) {
                    return self.narrow_type_by_typeof(f, t, &right, operator, &left, assume_true);
                }
                if self.is_matching_reference(f.reference.as_ref().unwrap(), &left) {
                    return self.narrow_type_by_equality(t, operator, &right, assume_true);
                }
                if self.is_matching_reference(f.reference.as_ref().unwrap(), &right) {
                    return self.narrow_type_by_equality(t, operator, &left, assume_true);
                }
                if let Some(access) = self.get_discriminant_property_access(f, &left, t) {
                    return self.narrow_type_by_discriminant_property(t, &access, operator, &right, assume_true);
                }
                if let Some(access) = self.get_discriminant_property_access(f, &right, t) {
                    return self.narrow_type_by_discriminant_property(t, &access, operator, &left, assume_true);
                }
                if is_identifier(&right) && right.text() == "true" || right.kind == SyntaxKind::TrueKeyword {
                    if !is_element_access_expression(&left) && !tsox_frontend::ast::is_property_access_expression(&left) {
                        return self.narrow_type_by_boolean_comparison(f, t, &left, &right, operator, assume_true);
                    }
                }
                if left.kind == SyntaxKind::TrueKeyword || right.kind == SyntaxKind::TrueKeyword || left.kind == SyntaxKind::FalseKeyword || right.kind == SyntaxKind::FalseKeyword {
                    if left.kind == SyntaxKind::TrueKeyword || left.kind == SyntaxKind::FalseKeyword {
                        if !is_element_access_expression(&right) && !tsox_frontend::ast::is_property_access_expression(&right) {
                            return self.narrow_type_by_boolean_comparison(f, t, &right, &left, operator, assume_true);
                        }
                    }
                }
                Arc::clone(t)
            }
            SyntaxKind::CommaToken => self.narrow_type(f, t, &expr.right, assume_true),
            SyntaxKind::AmpersandAmpersandToken => {
                if assume_true {
                    let inner = self.narrow_type(f, t, &expr.left, true);
                    self.narrow_type(f, &inner, &expr.right, true)
                } else {
                    let l = self.narrow_type(f, t, &expr.left, false);
                    let r = self.narrow_type(f, t, &expr.right, false);
                    self.get_union_type(vec![l, r])
                }
            }
            SyntaxKind::BarBarToken => {
                if assume_true {
                    let l = self.narrow_type(f, t, &expr.left, true);
                    let r = self.narrow_type(f, t, &expr.right, true);
                    self.get_union_type(vec![l, r])
                } else {
                    let inner = self.narrow_type(f, t, &expr.left, false);
                    self.narrow_type(f, &inner, &expr.right, false)
                }
            }
            _ => Arc::clone(t),
        }
    }

    pub fn narrow_type_by_boolean_comparison(
        &mut self,
        f: &mut FlowState,
        t: &Arc<Type>,
        expr: &Arc<Node>,
        bool_value: &Arc<Node>,
        operator: SyntaxKind,
        assume_true: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_boolean_comparison"); 
        let assume_true =
            (assume_true != (bool_value.kind == SyntaxKind::TrueKeyword)) != (operator != SyntaxKind::ExclamationEqualsEqualsToken && operator != SyntaxKind::ExclamationEqualsToken);
        self.narrow_type(f, t, expr, assume_true)
    }

    pub fn narrow_type_by_discriminant_property(
        &mut self,
        t: &Arc<Type>,
        access: &Arc<Node>,
        operator: SyntaxKind,
        value: &Arc<Node>,
        assume_true: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_discriminant_property"); 
        if (operator == SyntaxKind::EqualsEqualsEqualsToken || operator == SyntaxKind::ExclamationEqualsEqualsToken)
            && t.flags.intersects(TypeFlags::Union)
        {
            if let Some(key_property_name) = self.get_key_property_name(t) {
                if !key_property_name.is_empty()
                    && self.get_accessed_property_name(access).is_some_and(|n| n == key_property_name)
                {
                    let key_type = self.check_expression_ex(value, CheckMode::Normal);
                    let candidate = self.get_constituent_type_for_key_type(t, &key_type);
                    if let Some(candidate) = candidate {
                        if assume_true && operator == SyntaxKind::EqualsEqualsEqualsToken
                            || !assume_true && operator == SyntaxKind::ExclamationEqualsEqualsToken
                        {
                            return candidate;
                        }
                        if let Some(prop_type) = self.get_type_of_property_of_type(&candidate, &key_property_name) {
                            if crate::checker::utilities_token_is_identifier_or_keyword::is_unit_type(&prop_type) {
                                return self.remove_type(t, &candidate);
                            }
                        }
                        return Arc::clone(t);
                    }
                }
            }
        }
        self.narrow_type_by_discriminant(t, access, &|checker, t| checker.narrow_type_by_equality(t, operator, value, assume_true))
    }

    pub fn narrow_type_by_equality(&mut self, t: &Arc<Type>, operator: SyntaxKind, value: &Arc<Node>, assume_true: bool) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_equality"); 
        if t.flags.intersects(TypeFlags::Any) {
            return Arc::clone(t);
        }
        let assume_true = if operator == SyntaxKind::ExclamationEqualsToken || operator == SyntaxKind::ExclamationEqualsEqualsToken {
            !assume_true
        } else {
            assume_true
        };
        let value_type = self.check_expression_ex(value, CheckMode::Normal);
        let double_equals = operator == SyntaxKind::EqualsEqualsToken || operator == SyntaxKind::ExclamationEqualsToken;
        if value_type.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
            if !self.strict_null_checks {
                return Arc::clone(t);
            }
            let facts = if double_equals {
                if assume_true { TypeFacts::EQ_UNDEFINED_OR_NULL } else { TypeFacts::NE_UNDEFINED_OR_NULL }
            } else if value_type.flags.intersects(TypeFlags::Null) {
                if assume_true { TypeFacts::EQ_NULL } else { TypeFacts::NE_NULL }
            } else if assume_true {
                TypeFacts::EQ_UNDEFINED
            } else {
                TypeFacts::NE_UNDEFINED
            };
            return self.get_adjusted_type_with_facts(t, facts);
        }
        if assume_true {
            if !double_equals
                && (t.flags.intersects(TypeFlags::Unknown) || some_type(t, &|t: &Arc<Type>| self.is_empty_anonymous_object_type(t)))
            {
                if value_type.flags.intersects(TYPE_FLAGS_PRIMITIVE | TypeFlags::NonPrimitive)
                    || self.is_empty_anonymous_object_type(&value_type)
                {
                    return value_type;
                }
                if value_type.flags.intersects(TypeFlags::Object) {
                    return self.non_primitive_type();
                }
            }
            if !double_equals
                && value_type.flags.intersects(TYPE_FLAGS_PRIMITIVE)
                && self.is_uniform_union_type(t)
            {
                let regular_type = self.get_regular_type_of_literal_type(&value_type);
                if union_contains_type_w6(self, t, &regular_type) {
                    return regular_type;
                }
            }
            let filtered_type = filter_type_w6(self, t, &mut |checker, t: &Arc<Type>| {
                checker.are_types_comparable(t, &value_type)
                    || double_equals && Checker::is_coercible_under_double_equals(t, &value_type)
            });
            return self.replace_primitives_with_literals(&filtered_type, &value_type);
        }
        if is_unit_type(&value_type) {
            if self.is_uniform_union_type(t) {
                let filtered_type = self.remove_type(t, &self.get_regular_type_of_literal_type(&value_type));
                if !Arc::ptr_eq(&filtered_type, t) {
                    return filtered_type;
                }
            }
            return filter_type_w6(self, t, &mut |checker, t: &Arc<Type>| {
                !(checker.is_unit_like_type(t) && checker.are_types_comparable(t, &value_type))
            });
        }
        Arc::clone(t)
    }

    pub fn narrow_type_by_switch_on_discriminant(&mut self, t: &Arc<Type>, data_node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_switch_on_discriminant"); 
        let data = data_node.as_flow_switch_clause_data();
        let switch_types = self.get_switch_clause_types(&data.switch_statement);
        if switch_types.is_empty() {
            return Arc::clone(t);
        }
        let clause_types = &switch_types[data.clause_start as usize..data.clause_end as usize];
        let has_default_clause = data.clause_start == data.clause_end || clause_types.iter().any(|s| Arc::ptr_eq(s, self.never_type.get().unwrap()));
        if t.flags.intersects(TypeFlags::Unknown) && !has_default_clause {
            let mut ground_clause_types: Vec<Arc<Type>> = Vec::new();
            for s in clause_types {
                if s.flags.intersects(TYPE_FLAGS_PRIMITIVE | TypeFlags::NonPrimitive) {
                    ground_clause_types.push(Arc::clone(s));
                } else if s.flags.intersects(TypeFlags::Object) {
                    ground_clause_types.push(self.non_primitive_type());
                } else {
                    return Arc::clone(t);
                }
            }
            return self.get_union_type(ground_clause_types);
        }
        let discriminant_type = self.get_union_type(clause_types.to_vec());
        let mut case_type: Option<Arc<Type>> = None;
        if discriminant_type.flags.intersects(TypeFlags::Never) {
            case_type = Some(Arc::clone(self.never_type.get().unwrap()));
        } else {
            if discriminant_type.flags.intersects(TYPE_FLAGS_PRIMITIVE) && self.is_uniform_union_type(t) {
                let regular_type = self.get_regular_type_of_literal_type(&discriminant_type);
                if union_contains_type_w6(self, t, &regular_type) {
                    case_type = Some(regular_type);
                }
            }
            if case_type.is_none() {
                let filtered = filter_type_w6(self, t, &mut |checker, t: &Arc<Type>| checker.are_types_comparable(&discriminant_type, t));
                case_type = Some(self.replace_primitives_with_literals(&filtered, &discriminant_type));
            }
        }
        let case_type = case_type.unwrap();
        if !has_default_clause {
            return case_type;
        }
        let default_type = filter_type_w6(self, t, &mut |checker, t: &Arc<Type>| {
            if !checker.is_unit_like_type(t) {
                return true;
            }
            let u = if t.flags.intersects(TypeFlags::Undefined) {
                Arc::clone(checker.undefined_type.get().unwrap())
            } else {
                checker.get_regular_type_of_literal_type(&checker.extract_unit_type(t))
            };
            !switch_types
                .iter()
                .any(|st| is_unit_type(st) && checker.are_types_comparable(st, &u))
        });
        if case_type.flags.intersects(TypeFlags::Never) {
            return default_type;
        }
        self.get_union_type(vec![case_type, default_type])
    }

    pub fn narrow_type_by_switch_on_type_of(&mut self, t: &Arc<Type>, data_node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_switch_on_type_of"); 
        let data = data_node.as_flow_switch_clause_data();
        if let Some(witnesses) = self.get_switch_clause_type_of_witnesses(&data.switch_statement) {
            let clauses = switch_clauses(&data.switch_statement);
            let default_index = clauses.iter().position(|clause| clause.kind == SyntaxKind::DefaultClause);
            let clause_start = data.clause_start as usize;
            let clause_end = data.clause_end as usize;
            let has_default_clause =
                clause_start == clause_end || default_index.is_some_and(|i| i >= clause_start && i < clause_end);
            if has_default_clause {
                let not_equal_facts = self.get_not_equal_facts_from_typeof_switch(clause_start, clause_end, &witnesses);
                return filter_type_w6(self, t, &mut |checker, t: &Arc<Type>| {
                    checker.get_type_facts_worker(t, not_equal_facts) == not_equal_facts
                });
            }
            let clause_witnesses = &witnesses[clause_start..clause_end];
            let types = clause_witnesses
                .iter()
                .map(|text| {
                    if !text.is_empty() {
                        self.narrow_type_by_type_name(t, text)
                    } else {
                        Arc::clone(self.never_type.get().unwrap())
                    }
                })
                .collect::<Vec<_>>();
            return self.get_union_type(types);
        }
        Arc::clone(t)
    }

    pub fn narrow_type_by_switch_on_true(&mut self, f: &mut FlowState, t: &Arc<Type>, data_node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_switch_on_true"); 
        let data = data_node.as_flow_switch_clause_data();
        let clauses = switch_clauses(&data.switch_statement);
        let default_index = clauses.iter().position(|clause| clause.kind == SyntaxKind::DefaultClause);
        let clause_start = data.clause_start as usize;
        let clause_end = data.clause_end as usize;
        let has_default_clause =
            clause_start == clause_end || default_index.is_some_and(|i| i >= clause_start && i < clause_end);
        let mut t = Arc::clone(t);
        for clause in &clauses[..clause_start] {
            if clause.kind == SyntaxKind::CaseClause {
                t = self.narrow_type(f, &t, clause.expression().unwrap(), false);
            }
        }
        if has_default_clause {
            for clause in &clauses[clause_end..] {
                if clause.kind == SyntaxKind::CaseClause {
                    t = self.narrow_type(f, &t, clause.expression().unwrap(), false);
                }
            }
            return t;
        }
        let types = clauses[clause_start..clause_end]
            .iter()
            .map(|clause| {
                if clause.kind == SyntaxKind::CaseClause {
                    self.narrow_type(f, &t, clause.expression().unwrap(), true)
                } else {
                    Arc::clone(self.never_type.get().unwrap())
                }
            })
            .collect::<Vec<_>>();
        self.get_union_type(types)
    }

    pub fn get_switch_clause_type_of_witnesses(&mut self, node: &Arc<Node>) -> Option<Vec<String>> { ::tsox_core::fntrace::enter("get_switch_clause_type_of_witnesses"); 
        let clauses = switch_clauses(node);
        let mut witnesses = vec![String::new(); clauses.len()];
        for (i, clause) in clauses.iter().enumerate() {
            if clause.kind == SyntaxKind::CaseClause {
                let expr = clause.expression().unwrap();
                if !tsox_frontend::ast::is_string_literal_like(expr) {
                    return None;
                }
                let text = expr.text().to_string();
                if !witnesses.contains(&text) {
                    witnesses[i] = text;
                }
            }
        }
        Some(witnesses)
    }

    pub fn get_effects_signature(&mut self, node: &Arc<Node>) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("get_effects_signature"); 
        if let Some(links) = self.signature_links.get(node) {
            if let Some(signature) = &links.effects_signature {
                return Some(Arc::clone(signature));
            }
        }
        let mut func_type: Option<Arc<Type>> = None;
        if is_binary_expression(node) {
            let right_type = self.check_non_null_expression(&node.as_binary_expression().right);
            func_type = self.get_symbol_has_instance_method_of_object_type(&right_type);
        } else if node.parent().as_ref().is_some_and(|parent| tsox_frontend::ast::is_expression_statement(parent)) {
            func_type = self.get_type_of_dotted_name(&node.expression().unwrap().clone(), None);
        } else if node.expression().unwrap().kind != SyntaxKind::SuperKeyword {
            if is_optional_chain_w6(node) {
                let expression = node.expression().unwrap().clone();
                let expression_type = self.check_expression_ex(&expression, CheckMode::Normal);
                let optional_type = self.get_optional_expression_type(&expression_type, &expression);
                func_type = Some(self.check_non_null_type(&optional_type, &expression));
            } else {
                func_type = Some(self.check_non_null_expression(&node.expression().unwrap().clone()));
            }
        }
        let mut signature: Option<Arc<Signature>> = None;
        if let Some(func_type) = func_type {
            let apparent_type = self.get_apparent_type(&func_type);
            let signatures = self.get_signatures_of_type(&apparent_type, SignatureKind::Call);
            if signatures.len() == 1 && signatures[0].type_parameters.is_empty() {
                signature = Some(Arc::clone(&signatures[0]));
            } else if let Some(found) = signatures
                .iter()
                .find(|s| self.has_type_predicate_or_never_return_type(s))
            {
                signature = Some(Arc::clone(found));
            }
        }
        if signature
            .as_ref()
            .map(|s| !self.has_type_predicate_or_never_return_type(s))
            .unwrap_or(true)
        {
            signature = None;
        }
        self.signature_links.get_or_default(node).effects_signature = signature.clone();
        signature
    }

    fn unknown_signature_opt(&mut self) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("unknown_signature_opt"); 
        None
    }
}

fn switch_clauses(switch_statement: &Arc<Node>) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("switch_clauses"); 
    match &switch_statement.data {
        NodeData::SwitchStatement(d) => match &d.case_block.data {
            NodeData::CaseBlock(cb) => &cb.clauses.nodes,
            _ => &[],
        },
        _ => &[],
    }
}

fn every_type_is_tuple_like_w6(checker: &mut Checker, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("every_type_is_tuple_like_w6"); 
    if t.flags.intersects(TYPE_FLAGS_UNION) {
        t.types().unwrap_or(&[]).iter().all(|c| checker.is_tuple_like_type(c))
    } else {
        checker.is_tuple_like_type(t)
    }
}

fn some_type_w6(checker: &mut Checker, t: &Arc<Type>, f: &mut dyn FnMut(&mut Checker, &Arc<Type>) -> bool) -> bool { ::tsox_core::fntrace::enter("some_type_w6"); 
    if t.flags.intersects(TypeFlags::Union) {
        t.types().unwrap_or(&[]).iter().any(|c| f(checker, c))
    } else {
        f(checker, t)
    }
}

fn contains_missing_type_w6(checker: &Checker, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("contains_missing_type_w6"); 
    let missing = checker.missing_type();
    Arc::ptr_eq(t, &missing)
        || t.flags.intersects(TYPE_FLAGS_UNION) && t.types().is_some_and(|types| types.first().is_some_and(|first| Arc::ptr_eq(first, &missing)))
}

fn filter_type_w6(
    checker: &mut Checker,
    t: &Arc<Type>,
    f: &mut dyn FnMut(&mut Checker, &Arc<Type>) -> bool,
) -> Arc<Type> { ::tsox_core::fntrace::enter("filter_type_w6"); 
    if t.flags.intersects(TypeFlags::Union) {
        let types = t.types().unwrap_or(&[]).to_vec();
        let filtered: Vec<Arc<Type>> = types.into_iter().filter(|c| f(checker, c)).collect();
        return checker.get_union_type(filtered);
    }
    if f(checker, t) {
        Arc::clone(t)
    } else {
        Arc::clone(checker.never_type.get().unwrap())
    }
}
