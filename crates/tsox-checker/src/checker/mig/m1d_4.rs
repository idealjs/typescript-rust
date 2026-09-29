#![allow(unused_imports)]
#![allow(dead_code)]

pub(crate) use super::m1d::r21k7_defs::*;

use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{Node, NodeData, Symbol, SyntaxKind};
use tsox_frontend::ast::{
    is_static, is_expression, is_function_expression_or_arrow_function, is_object_literal_method,
    is_tagged_template_expression,
};

pub(crate) fn get_big_int_literal_value(t: &Arc<Type>) -> tsox_core::jsnum::PseudoBigInt {
    match t.literal_data().map(|d| &d.value) {
        Some(LiteralValue::BigInt(b)) => b.clone(),
        _ => unreachable!("getBigIntLiteralValue on non bigint literal"),
    }
}

pub(crate) fn get_boolean_literal_value(t: &Arc<Type>) -> bool {
    match t.literal_data().map(|d| &d.value) {
        Some(LiteralValue::Boolean(b)) => *b,
        _ => unreachable!("getBooleanLiteralValue on non boolean literal"),
    }
}

pub(crate) fn for_each_type(t: &Arc<Type>, f: &mut dyn FnMut(&Arc<Type>)) {
    if t.flags.contains(TypeFlags::Union) {
        if let Some(us) = t.types() {
            for u in us {
                f(u);
            }
        }
    } else {
        f(t);
    }
}

impl Checker {
    pub(crate) fn get_base_type_of_enum_like_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::EnumLiteral) {
            if let Some(symbol) = &t.symbol {
                if symbol.flags.contains(SymbolFlags::EnumMember) {
                    if let Some(parent) = self.get_parent_of_symbol(symbol) {
                        return self.get_declared_type_of_symbol(&parent);
                    }
                }
            }
        }
        Arc::clone(t)
    }

    pub(crate) fn get_base_type_of_literal_type_union(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let self_ptr = self as *mut Checker;
        self.map_type(t, &mut |t| {
            let ch = unsafe { &*self_ptr };
            Some(ch.get_base_type_of_literal_type(t))
        })
        .unwrap_or_else(|| Arc::clone(t))
    }

    pub(crate) fn get_contextual_type_for_variable_like_declaration(
        &mut self,
        declaration: &Arc<Node>,
        context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        if let Some(type_node) = declaration.type_node() {
            return Some(self.get_type_from_type_node(type_node));
        }
        match declaration.kind {
            SyntaxKind::Parameter => self.get_contextually_typed_parameter_type(declaration),
            SyntaxKind::BindingElement => self
                .get_contextual_type_for_binding_element(declaration, context_flags),
            SyntaxKind::PropertyDeclaration => {
                if is_static(declaration) {
                    return self
                        .get_contextual_type_for_static_property_declaration(
                            declaration,
                            context_flags,
                        );
                }
                None
            }
            _ => None,
        }
    }

    pub(crate) fn get_contextual_type_for_static_property_declaration(
        &mut self,
        declaration: &Arc<Node>,
        context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        if let Some(parent) = declaration.parent() {
            if is_expression(&parent) {
                if let Some(parent_type) = self.get_contextual_type(&parent, context_flags) {
                    let symbol = self.get_symbol_of_declaration(declaration)?;
                    return self.get_type_of_property_of_contextual_type(
                        &parent_type,
                        &symbol.name,
                    );
                }
            }
        }
        None
    }

    pub(crate) fn get_contextual_signature_for_function_like_declaration(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Signature>> {
        if is_function_expression_or_arrow_function(node) || is_object_literal_method(node) {
            return self.get_contextual_signature(node);
        }
        None
    }

    pub(crate) fn get_contextual_type_for_conditional_operand(
        &mut self,
        node: &Arc<Node>,
        context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        if let Some(parent) = node.parent() {
            if let NodeData::ConditionalExpression(conditional) = &parent.data {
                let is_branch = conditional.when_true.id() == node.id()
                    || conditional.when_false.id() == node.id();
                if is_branch {
                    return self.get_contextual_type(&parent, context_flags);
                }
            }
        }
        None
    }

    pub(crate) fn get_contextual_type_for_substitution_expression(
        &mut self,
        template: &Arc<Node>,
        substitution_expression: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        if let Some(parent) = template.parent() {
            if is_tagged_template_expression(&parent) {
                return self.get_contextual_type_for_argument(&parent, substitution_expression);
            }
        }
        None
    }

    pub(crate) fn get_contextual_import_attribute_type(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        let global_type = self.get_global_import_attributes_type();
        let name = node.name().map(|n| n.text().to_string()).unwrap_or_default();
        self.get_type_of_property_of_contextual_type(&global_type, &name)
    }

    pub(crate) fn get_effective_decorator_arguments(&mut self, node: &Arc<Node>) -> Vec<Arc<Node>> {
        let expr = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let signature = self.get_decorator_call_signature(node);
        if let Some(signature) = signature {
            let mut args = Vec::with_capacity(signature.parameters.len());
            for param in &signature.parameters {
                let t = self.get_type_of_symbol(param);
                args.push(
                    self.create_synthetic_expression(&expr, &t, false, None),
                );
            }
            return args;
        }
        unreachable!("Decorator signature not found")
    }
}
