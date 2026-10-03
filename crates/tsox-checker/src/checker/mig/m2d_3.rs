#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use crate::checker::types::{
    CacheHashKey, NodeCheckFlags, Signature, Type, TypeFlags, TypePredicate, TypePredicateKind,
};
use std::sync::Arc;
use crate::checker::utilities_get_assignment_target::get_binding_element_property_name;
use crate::checker::utilities_has_only_expression_initialization::is_empty_array_literal;
use crate::checker::mig::m2e::r19k3_defs::any_to_string;
use crate::checker::mig::m2h::r22k10_defs::flow_node_of;
use crate::checker::utilities_token_is_identifier_or_keyword::{
    get_property_name_from_type, is_type_usable_as_property_name,
};
use tsox_frontend::ast::mig::m3e::FlowReduceLabelData;
use tsox_frontend::ast::mig::m3g_3::skip_parentheses;
use tsox_frontend::ast::{
    find_ancestor, is_access_expression, is_array_binding_pattern, is_array_literal_expression,
    is_binding_element, is_class_like, is_function_like,
    is_function_or_source_file, is_identifier, is_object_binding_pattern, is_parameter_declaration,
    is_property_declaration, is_property_signature_declaration, is_shorthand_property_assignment,
    is_property_assignment, is_static, is_this_in_type_query, is_variable_declaration, Node,
    NodeData, Symbol, SyntaxKind,
};

pub struct FlowType {
    pub t: Option<Arc<Type>>,
    pub incomplete: bool,
}

pub struct FlowState {
    pub reference: Option<Arc<Node>>,
    pub declared_type: Option<Arc<Type>>,
    pub initial_type: Option<Arc<Type>>,
    pub flow_container: Option<Arc<Node>>,
    pub ref_key: CacheHashKey,
    pub depth: i32,
    pub shared_flow_start: i32,
    pub reduce_labels: Vec<Arc<FlowReduceLabelData>>,
    pub next: Option<Box<FlowState>>,
}

pub fn get_flow_node_of_node(node: &Arc<Node>) -> Option<Arc<tsox_frontend::ast::FlowNode>> { ::tsox_core::fntrace::enter("get_flow_node_of_node"); 
    flow_node_of(node)
}

pub fn is_evolving_array_type_list(types: &[Arc<Type>]) -> bool { ::tsox_core::fntrace::enter("is_evolving_array_type_list"); 
    let mut has_evolving_array_type = false;
    for t in types {
        if !t.flags.intersects(TypeFlags::Never) {
            if !t
                .object_flags
                .intersects(crate::checker::types::ObjectFlags::EvolvingArray)
            {
                return false;
            }
            has_evolving_array_type = true;
        }
    }
    has_evolving_array_type
}

impl FlowType {
    pub fn is_nil(&self) -> bool { ::tsox_core::fntrace::enter("is_nil"); 
        self.t.is_none()
    }
}

impl Checker {
    pub fn get_flow_state(&mut self) -> Box<FlowState> { ::tsox_core::fntrace::enter("get_flow_state"); 
        Box::new(FlowState {
            reference: None,
            declared_type: None,
            initial_type: None,
            flow_container: None,
            ref_key: Default::default(),
            depth: 0,
            shared_flow_start: 0,
            reduce_labels: Vec::new(),
            next: None,
        })
    }

    pub fn put_flow_state(&mut self, state: Box<FlowState>) { ::tsox_core::fntrace::enter("put_flow_state"); 
        drop(state);
    }

    pub fn is_empty_array_assignment(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_empty_array_assignment"); 
        if is_variable_declaration(node) {
            if let Some(initializer) = node.initializer() {
                if is_empty_array_literal(&initializer) {
                    return true;
                }
            }
        }
        if !is_binding_element(node) {
            if let Some(parent) = node.parent() {
                if parent.kind == SyntaxKind::BinaryExpression {
                    let NodeData::BinaryExpression(binary) = &parent.data else {
                        return false;
                    };
                    return is_empty_array_literal(&binary.right);
                }
            }
        }
        false
    }

    pub fn get_destructuring_property_name(&mut self, node: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("get_destructuring_property_name"); 
        let parent = node.parent()?;
        if is_binding_element(node) && is_object_binding_pattern(&parent) {
            let property_name = get_binding_element_property_name(node)?;
            return self.get_literal_property_name_text(&property_name);
        }
        if is_property_assignment(node) || is_shorthand_property_assignment(node) {
            let name = node.name()?;
            return self.get_literal_property_name_text(name);
        }
        if is_array_literal_expression(&parent) || is_array_binding_pattern(&parent) {
            let index = parent
                .elements()?
                .nodes
                .iter()
                .position(|element| Arc::ptr_eq(element, node))?;
            return Some(index.to_string());
        }
        None
    }

    pub fn get_literal_property_name_text(&mut self, name: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("get_literal_property_name_text"); 
        let t = self.get_literal_type_from_property_name(name)?;
        if t.flags.intersects(TypeFlags::StringLiteral | TypeFlags::NumberLiteral) {
            if let Some(lit) = t.as_literal_type() {
                return Some(any_to_string(&lit.value));
            }
        }
        None
    }

    pub fn is_constant_reference(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_constant_reference"); 
        match node.kind {
            SyntaxKind::ThisKeyword => true,
            SyntaxKind::Identifier => {
                if !is_this_in_type_query(node) {
                    if let Some(symbol) = self.get_resolved_symbol(node) {
                        return self.is_constant_variable(&symbol)
                            || self.is_parameter_or_mutable_local_variable(&symbol)
                                && !self.is_symbol_assigned(&symbol)
                            || symbol
                                .value_declaration
                                .as_ref()
                                .is_some_and(|d| tsox_frontend::ast::is_function_expression(d));
                    }
                }
                false
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                if let Some(expression) = node.expression() {
                    if self.is_constant_reference(&expression) {
                        if let Some(symbol) = self.get_resolved_symbol_or_nil(node) {
                            return self.is_readonly_symbol(&symbol);
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn has_matching_argument(&mut self, expression: &Arc<Node>, reference: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_matching_argument"); 
        let arguments = expression.arguments();
        let argument_nodes: &[Arc<Node>] = match &arguments {
            Some(list) => &list.nodes,
            None => &[],
        };
        for argument in argument_nodes {
            if self.is_or_contains_matching_reference(reference, argument)
                || self.optional_chain_contains_reference(argument, reference)
            {
                return true;
            }
        }
        if let Some(expr) = expression.expression() {
            if expr.kind == SyntaxKind::PropertyAccessExpression {
                if let Some(expr_expression) = expr.expression() {
                    if self.is_or_contains_matching_reference(reference, &expr_expression) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_or_contains_matching_reference(
        &mut self,
        source: &Arc<Node>,
        target: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("is_or_contains_matching_reference"); 
        self.is_matching_reference(source, target) || self.contains_matching_reference(source, target)
    }

    pub fn get_property_name_for_known_symbol_name(&mut self, symbol_name: &str) -> String { ::tsox_core::fntrace::enter("get_property_name_for_known_symbol_name"); 
        if let Some(ctor_type) = self.get_global_es_symbol_constructor_type_symbol_or_nil() {
            let ctor_type_of = self.get_type_of_symbol(&ctor_type);
            if let Some(unique_type) = self.get_type_of_property_of_type(&ctor_type_of, symbol_name)
            {
                if is_type_usable_as_property_name(&unique_type) {
                    return get_property_name_from_type(&unique_type);
                }
            }
        }
        format!("{}@{}", tsox_frontend::ast::INTERNAL_SYMBOL_NAME_PREFIX, symbol_name)
    }

    pub fn is_declaration_with_explicit_type_annotation(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_declaration_with_explicit_type_annotation"); 
        (is_variable_declaration(node)
            || is_property_declaration(node)
            || is_property_signature_declaration(node)
            || is_parameter_declaration(node))
            && node.type_().is_some()
            || self.is_expando_property_function_with_return_type_annotation(node)
    }

    pub fn is_expando_property_function_with_return_type_annotation(
        &self,
        node: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("is_expando_property_function_with_return_type_annotation"); 
        if node.kind == SyntaxKind::BinaryExpression {
            let NodeData::BinaryExpression(binary) = &node.data else {
                return false;
            };
            if is_function_like(&binary.right) && binary.right.type_().is_some() {
                return true;
            }
        }
        false
    }

    pub fn has_type_predicate_or_never_return_type(&mut self, sig: &Arc<Signature>) -> bool { ::tsox_core::fntrace::enter("has_type_predicate_or_never_return_type"); 
        if self.get_type_predicate_of_signature(sig).is_some() {
            return true;
        }
        if let Some(declaration) = &sig.declaration {
            let return_type = self.get_return_type_from_annotation(declaration);
            return return_type.flags.intersects(TypeFlags::Never);
        }
        false
    }

    pub fn get_explicit_this_type(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_explicit_this_type"); 
        let container = crate::checker::checker_this_container::get_this_container(node, false, false);
        if is_function_like(&container) {
            if let Some(signature) = self.get_signature_from_declaration(&container) {
                if let Some(this_parameter) = &signature.this_parameter {
                    return self.get_explicit_type_of_symbol(this_parameter, None);
                }
            }
        }
        if let Some(parent) = container.parent() {
            if is_class_like(&parent) {
                let symbol = self.get_symbol_of_declaration(&parent)?;
                if is_static(&container) {
                    return Some(self.get_type_of_symbol(&symbol));
                }
                let declared = self.get_declared_type_of_symbol(&symbol);
                return declared.as_interface_type().and_then(|it| it.this_type.clone());
            }
        }
        None
    }

    pub fn include_undefined_in_index_signature(&mut self, t: Option<&Arc<Type>>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("include_undefined_in_index_signature"); 
        let t = t?;
        if self.no_unchecked_indexed_access {
            return Some(
                self.get_union_type(vec![Arc::clone(t), Arc::clone(&self.missing_type)]),
            );
        }
        Some(Arc::clone(t))
    }

    pub fn get_type_with_default(
        &mut self,
        t: &Arc<Type>,
        default_expression: Option<&Arc<Node>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_with_default"); 
        if let Some(default_expression) = default_expression {
            let non_undefined = self.get_non_undefined_type(t);
            let default_type = self.get_type_of_expression(default_expression);
            return self.get_union_type(vec![non_undefined, default_type]);
        }
        Arc::clone(t)
    }

    pub fn get_type_predicate_argument(
        &self,
        predicate: &TypePredicate,
        call_expression: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_type_predicate_argument"); 
        if predicate.kind == TypePredicateKind::Identifier
            || predicate.kind == TypePredicateKind::AssertsIdentifier
        {
            if let Some(arguments) = call_expression.arguments() {
                if predicate.parameter_index >= 0
                    && (predicate.parameter_index as usize) < arguments.nodes.len()
                {
                    return Some(Arc::clone(
                        &arguments.nodes[predicate.parameter_index as usize],
                    ));
                }
            }
        } else {
            let invoked_expression = skip_parentheses(call_expression.expression()?);
            if is_access_expression(&invoked_expression) {
                return Some(skip_parentheses(invoked_expression.expression()?));
            }
        }
        None
    }
}
