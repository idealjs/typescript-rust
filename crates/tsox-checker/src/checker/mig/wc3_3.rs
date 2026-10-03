#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use crate::checker::mig::wc2_2::get_mapped_type_modifiers;
use crate::checker::mig::wc3::{MappedTypeModifiers, TYPE_FLAGS_INSTANTIABLE, NodeAccessExt};
use crate::checker::mig::wc1c::r23k4_defs::R23k4NodeExt;
use crate::checker::utilities_token_is_identifier_or_keyword::is_tuple_type;
use crate::checker::utilities_is_optional_symbol::is_type_alias;
use crate::checker::mig::m1d_4::get_big_int_literal_value;
use crate::checker::mig::wc3::is_node_descendant_of;
use tsox_core::jsnum;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as msg;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn is_for_in_variable_for_numeric_property_names(&mut self, expr: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_for_in_variable_for_numeric_property_names"); 
        let e = ast::skip_parentheses(expr);
        if ast::is_identifier(&e) {
            if let Some(symbol) = self.get_resolved_symbol(&e) {
                if symbol.flags.intersects(SymbolFlags::VARIABLE) {
                let mut child = Arc::clone(expr);
                let mut node = expr.parent();
                while let Some(current) = node {
                    if ast::is_for_in_statement(&current)
                        && Arc::ptr_eq(
                            &current.as_for_in_or_of_statement().statement,
                            &child,
                        )
                        && self
                            .get_for_in_variable_symbol(&current)
                            .as_ref()
                            .is_some_and(|s| Arc::ptr_eq(s, &symbol))
                        && {
                            let expression_type =
                                self.get_type_of_expression(&current.expression().unwrap());
                            self.has_numeric_property_names(&expression_type)
                        }
                    {
                        return true;
                    }
                    node = current.parent();
                    child = current;
                }
                }
            }
        }
        false
    }

    pub fn is_function_type(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_function_type"); 
        t.flags.intersects(TypeFlags::Object)
            && !self.get_signatures_of_type(t, SignatureKind::Call).is_empty()
    }

    pub fn is_generic_function_returning_function(&mut self, signature: &Arc<Signature>) -> bool { ::tsox_core::fntrace::enter("is_generic_function_returning_function"); 
        !signature.type_parameters.is_empty()
            && self.is_function_type(
                self.get_return_type_of_signature(signature)
                    .as_ref()
                    .unwrap(),
            )
    }

    pub fn is_generic_object_type(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_generic_object_type"); 
        self.get_generic_object_flags(t)
            .intersects(ObjectFlags::IsGenericObjectType)
    }

    pub fn is_generic_reducible_type(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_generic_reducible_type"); 
        (t.flags.intersects(TypeFlags::Union)
            && t.object_flags.intersects(ObjectFlags::ContainsIntersections)
            && t.types()
                .unwrap()
                .iter()
                .any(|x| self.is_generic_reducible_type(x)))
            || (t.flags.intersects(TypeFlags::Intersection) && self.is_reducible_intersection(t))
    }

    pub fn is_generic_string_like_type(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_generic_string_like_type"); 
        t.flags
            .intersects(TypeFlags::TemplateLiteral | TypeFlags::StringMapping)
            && !self.is_pattern_literal_type(t)
    }

    pub fn is_generic_type_with_undefined_constraint(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_generic_type_with_undefined_constraint"); 
        if t.flags.intersects(TYPE_FLAGS_INSTANTIABLE) {
            let constraint = self.get_base_constraint_of_type(t);
            if let Some(constraint) = constraint {
                return self.maybe_type_of_kind(&constraint, TypeFlags::Undefined);
            }
        }
        false
    }

    pub fn is_generic_type_without_nullable_constraint(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_generic_type_without_nullable_constraint"); 
        if t.flags.contains(TypeFlags::Intersection) {
            return t
                .as_intersection_type()
                .unwrap()
                .union_or_intersection
                .types
                .iter()
                .any(|x| self.is_generic_type_without_nullable_constraint(x));
        }
        t.flags.intersects(TYPE_FLAGS_INSTANTIABLE)
            && !self.maybe_type_of_kind(&self.get_base_constraint_or_type(t), crate::checker::types_type_id::TYPE_FLAGS_NULLABLE)
    }

    pub fn is_global_nan(&mut self, expr: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_global_nan"); 
        if ast::is_identifier(expr) && expr.text() == "NaN" {
            let global_nan_symbol =
                crate::checker::mig::wc3::r24k3_defs::get_global_nan_symbol_or_nil(self);
            return global_nan_symbol.is_some()
                && self
                    .get_resolved_symbol(expr)
                    .as_ref()
                    .is_some_and(|s| {
                        Arc::ptr_eq(s, global_nan_symbol.as_ref().unwrap())
                    });
        }
        false
    }

    pub fn is_global_symbol_constructor(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_global_symbol_constructor"); 
        let symbol = self.get_symbol_of_node(node);
        let global_symbol = self.get_global_es_symbol_constructor_type_symbol_or_nil();
        global_symbol.is_some() && symbol.is_some()
            && Arc::ptr_eq(symbol.as_ref().unwrap(), global_symbol.as_ref().unwrap())
    }

    pub fn is_implementation_compatible_with_overload(
        &mut self,
        implementation: &Arc<Signature>,
        overload: &Arc<Signature>,
    ) -> bool { ::tsox_core::fntrace::enter("is_implementation_compatible_with_overload"); 
        let erased_source = self.get_erased_signature(implementation);
        let erased_target = self.get_erased_signature(overload);
        let source_return_type = self.get_return_type_of_signature(&erased_source);
        let target_return_type = self.get_return_type_of_signature(&erased_target);
        if Arc::ptr_eq(
            target_return_type.as_ref().unwrap(),
            self.void_type.get().unwrap(),
        ) || self.is_type_related_to(
            target_return_type.as_ref().unwrap(),
            source_return_type.as_ref().unwrap(),
            self.assignable_relation().kind,
        ) || self.is_type_related_to(
            source_return_type.as_ref().unwrap(),
            target_return_type.as_ref().unwrap(),
            self.assignable_relation().kind,
        ) {
            return self.is_signature_assignable_to(&erased_source, &erased_target, true);
        }
        false
    }

    pub fn is_inline_import_attributes(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_inline_import_attributes"); 
        if !ast::is_object_literal_expression(node)
            || !node
                .parent()
                .as_ref()
                .is_some_and(|p| ast::is_property_assignment(p))
            || !Arc::ptr_eq(
                &node.parent().unwrap().initializer().unwrap(),
                node,
            )
        {
            return false;
        }
        let property = node.parent().unwrap();
        let property_name = property.name().unwrap();
        if (!ast::is_identifier(&property_name) && !ast::is_string_literal_like(&property_name))
            || property_name.text() != "with"
        {
            return false;
        }
        let options = property.parent().unwrap();
        if !ast::is_object_literal_expression(&options) {
            return false;
        }
        let import_call = ast::find_ancestor(&options, |n: &Node| ast::is_import_call(n));
        import_call.is_some()
            && import_call
                .as_ref()
                .unwrap()
                .arguments()
                .as_ref()
                .is_some_and(|a| a.nodes.len() > 1)
            && Arc::ptr_eq(
                &ast::skip_parentheses(
                    &import_call
                        .as_ref()
                        .unwrap()
                        .arguments()
                        .as_ref()
                        .unwrap()
                        .nodes[1],
                ),
                &options,
            )
    }

    pub fn is_intersection_empty(&mut self, type1: &Arc<Type>, type2: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_intersection_empty"); 
        let intersected = self.intersect_types(Some(type1), Some(type2));
        let never_type = Arc::clone(self.never_type.get().unwrap());
        self.get_union_type(vec![intersected.unwrap(), never_type])
            .flags
            .contains(TypeFlags::Never)
    }

    pub fn is_key_type_included(&mut self, key_type: &Arc<Type>, include: TypeFlags) -> bool { ::tsox_core::fntrace::enter("is_key_type_included"); 
        key_type.flags.intersects(include)
            || key_type.flags.intersects(TypeFlags::Intersection)
                && key_type
                    .types()
                    .into_iter()
                    .flatten()
                    .any(|t| self.is_key_type_included(t, include))
    }

    pub fn is_late_bindable_index_signature(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_late_bindable_index_signature"); 
        if !is_late_bindable_ast(node) {
            return false;
        }
        if ast::is_computed_property_name(node) {
            let computed_type = self.check_computed_property_name_type(node);
            return self.is_type_usable_as_index_signature_declaration(&computed_type);
        }
        let argument_type = self.check_expression_cached(
            &node.as_element_access_expression().argument_expression,
        );
        self.is_type_usable_as_index_signature_declaration(&argument_type)
    }

    pub fn is_mapped_type_generic_indexed_access(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_mapped_type_generic_indexed_access"); 
        if t.flags.intersects(TypeFlags::IndexedAccess) {
            let indexed_access = t.as_indexed_access_type().unwrap();
            let object_type = indexed_access.object_type.as_ref().unwrap();
            return object_type.object_flags.intersects(ObjectFlags::Mapped)
                && !self.is_generic_mapped_type(object_type)
                && self.is_generic_index_type(indexed_access.index_type.as_ref().unwrap())
                && get_mapped_type_modifiers(object_type)
                    .intersects(MappedTypeModifiers::ExcludeOptional)
                    == false
                && object_type
                    .as_mapped_type()
                    .unwrap()
                    .declaration
                    .as_ref()
                    .map(|d| d.as_mapped_type_node())
                    .and_then(|m| m.name_type.as_ref())
                    .is_none();
        }
        false
    }

    pub fn is_method_access_for_call(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_method_access_for_call"); 
        let mut node = Arc::clone(node);
        while node
            .parent()
            .as_ref()
            .is_some_and(|p| ast::is_parenthesized_expression(p))
        {
            node = node.parent().unwrap();
        }
        node.parent()
            .as_ref()
            .is_some_and(|p| ast::mig::m3f_4::is_call_or_new_expression(p))
            && Arc::ptr_eq(&node.parent().unwrap().expression().unwrap(), &node)
    }

    pub fn is_mutable_array_or_tuple(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_mutable_array_or_tuple"); 
        (self.is_array_type(t) && !self.is_readonly_array_type(t))
            || (is_tuple_type(t) && !t.target_tuple_type().unwrap().readonly)
    }

    pub fn is_type_parameter_possibly_referenced(
        &mut self,
        tp: &Arc<Type>,
        node: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("is_type_parameter_possibly_referenced"); 
        if tp.symbol().is_some() && tp.symbol().as_ref().unwrap().declarations.len() == 1 {
            let container = tp
                .symbol()
                .as_ref()
                .unwrap()
                .declarations[0]
                .parent()
                .unwrap();
            let mut n = Some(Arc::clone(node));
            while let Some(current) = n {
                if Arc::ptr_eq(&current, &container) {
                    break;
                }
                if ast::is_block(&current)
                    || (ast::is_conditional_type_node(&current)
                        && type_parameter_contains_reference(self, tp, &current.as_conditional_type_node().extends_type))
                {
                    return true;
                }
                n = current.parent();
            }
            return type_parameter_contains_reference(self, tp, node);
        }
        true
    }
}

fn type_parameter_contains_reference(c: &mut Checker, tp: &Arc<Type>, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("type_parameter_contains_reference"); 
    match node.kind {
        SyntaxKind::ThisType => tp.as_type_parameter().unwrap().is_this_type,
        SyntaxKind::TypeReference => {
            if !tp.as_type_parameter().unwrap().is_this_type
                && node.type_arguments().map_or(true, |x| x.is_empty())
                && c
                    .get_symbol_from_type_reference(node)
                    .as_ref()
                    .is_some_and(|s| Arc::ptr_eq(s, tp.symbol().as_ref().unwrap()))
            {
                return true;
            }
            node.for_each_child(|child| {
                if type_parameter_contains_reference(c, tp, child) {
                    return true;
                }
                false
            })
        }
        SyntaxKind::TypeQuery => {
            let entity_name = node.as_type_query_node().expr_name.clone();
            let first_identifier = ast::mig::m3e_4::get_first_identifier(&entity_name);
            if !ast::is_this_identifier(Some(first_identifier.as_ref())) {
                let first_identifier_symbol = c.get_resolved_symbol(&first_identifier);
                let tp_declaration = tp.symbol().as_ref().unwrap().declarations[0].clone();
                let mut tp_scope: Option<Arc<Node>> = None;
                if ast::is_type_parameter_declaration(&tp_declaration) {
                    tp_scope = tp_declaration.parent();
                } else if tp.as_type_parameter().unwrap().is_this_type {
                    tp_scope = Some(tp_declaration);
                }
                if let Some(tp_scope) = tp_scope {
                    let declarations: Vec<Arc<Node>> = first_identifier_symbol
                        .as_ref()
                        .unwrap()
                        .declarations
                        .clone();
                    return declarations
                        .iter()
                        .any(|d| is_node_descendant_of(d, &tp_scope))
                        || node
                            .type_arguments()
                            .is_some_and(|l| {
                                l.iter()
                                    .any(|arg| type_parameter_contains_reference(c, tp, arg))
                            });
                }
            }
            true
        }
        SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => {
            let return_type = node.typ();
            return_type.is_none() && node.body().is_some()
                || node.type_parameters().is_some_and(|l| {
                    l.iter().any(|p| type_parameter_contains_reference(c, tp, p))
                })
                || node.parameters().is_some_and(|l| {
                    l.iter().any(|p| type_parameter_contains_reference(c, tp, p))
                })
                || return_type
                    .as_ref()
                    .is_some_and(|rt| type_parameter_contains_reference(c, tp, rt))
        }
        _ => node.for_each_child(|child| {
            if type_parameter_contains_reference(c, tp, child) {
                return true;
            }
            false
        }),
    }
}

pub fn is_generic_tuple_type(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_generic_tuple_type"); 
    is_tuple_type(t) && t.target_tuple_type().unwrap().combined_flags.intersects(ElementFlags::Variadic)
}

pub fn is_identifier_that_starts_with_underscore(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_identifier_that_starts_with_underscore"); 
    ast::is_identifier(node) && !node.text().is_empty() && node.text().starts_with('_')
}

pub fn is_immediately_used_in_initializer_of_block_scoped_variable(
    declaration: &Arc<Node>,
    usage: &Arc<Node>,
    decl_container: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("is_immediately_used_in_initializer_of_block_scoped_variable"); 
    let grandparent = declaration.parent().unwrap().parent().unwrap();
    match grandparent.kind {
        SyntaxKind::VariableStatement | SyntaxKind::ForStatement | SyntaxKind::ForOfStatement => {
            if is_same_scope_descendent_of(usage, declaration, decl_container) {
                return true;
            }
        }
        _ => {}
    }
    ast::is_for_in_or_of_statement(&grandparent)
        && is_same_scope_descendent_of(
            usage,
            &grandparent.expression().unwrap(),
            decl_container,
        )
}

fn is_same_scope_descendent_of(
    initial: &Arc<Node>,
    parent: &Arc<Node>,
    stop_at: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("is_same_scope_descendent_of"); 
    let mut cur = Some(Arc::clone(initial));
    while let Some(n) = cur {
        if Arc::ptr_eq(&n, parent) {
            return true;
        }
        if Arc::ptr_eq(&n, stop_at)
            || (ast::is_function_like(&n)
                && (ast::mig::m3e_4::get_immediately_invoked_function_expression(&n).is_none()
                    || ast::mig::m3e::get_function_flags(Some(&n))
                        .contains(ast::mig::m3e::FunctionFlags::ASYNC_GENERATOR)))
        {
            return false;
        }
        cur = n.parent();
    }
    false
}

pub fn is_instance_property_with_initializer_or_private_identifier_property(n: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_instance_property_with_initializer_or_private_identifier_property"); 
    tsox_frontend::ast::mig::m3g_2::is_private_identifier_class_element_declaration(n)
        || (ast::is_property_declaration(n)
            && !ast::is_static(n)
            && n.initializer().is_some())
}

pub fn is_internal_module_import_equals_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_internal_module_import_equals_declaration"); 
    node.kind == SyntaxKind::ImportEqualsDeclaration
        && node.as_import_equals_declaration().module_reference.kind
            != SyntaxKind::ExternalModuleReference
}

pub fn is_intersection_type(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_intersection_type"); 
    t.flags.intersects(TypeFlags::Intersection)
}

pub fn is_invalid_computed_property_name(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_invalid_computed_property_name"); 
    let grandparent = node.parent().unwrap().parent().unwrap();
    (ast::is_type_literal_node(&grandparent)
        || ast::is_class_like(&grandparent)
        || ast::is_interface_declaration(&grandparent))
        && ast::is_binary_expression(&node.expression().unwrap())
        && node
            .expression()
            .unwrap()
            .as_binary_expression()
            .operator_token
            .kind
            == SyntaxKind::InKeyword
        && !ast::is_accessor(&node.parent().unwrap())
}

pub fn is_late_bindable_ast(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_late_bindable_ast"); 
    let expr = if ast::is_computed_property_name(node) {
        node.expression()
    } else if ast::is_element_access_expression(node) {
        Some(&node.as_element_access_expression().argument_expression)
    } else {
        None
    };
    expr.is_some() && ast::is_entity_name_expression(&expr.unwrap())
}

pub fn is_local_type_alias(symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_local_type_alias"); 
    let declaration = symbol
        .declarations
        .iter()
        .find(|d| is_type_alias(d))
        .cloned();
    declaration.is_some()
        && ast::mig::x4ast::get_containing_function(&declaration.unwrap()).is_some()
}

pub fn is_zero_big_int(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_zero_big_int"); 
    get_big_int_literal_value(t) == jsnum::PseudoBigInt::new("0", false)
}
