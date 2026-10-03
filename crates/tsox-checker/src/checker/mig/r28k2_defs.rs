#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::checker_iteration::IterationUse;
use crate::checker::mig::m2f::NodeBuilder;
use crate::checker::mig::m2b::r22k6_defs::R22K6NodeBuilderExt as _;
use crate::checker::mig::wc1b::{every_type, is_type_any, is_tuple_type};
use crate::checker::symboltracker::{NodeBuilderFlags, NodeBuilderInternalFlags};
use crate::checker::types_type_flags_instantiable_non_primitive::AccessFlags;
use std::sync::{Arc, OnceLock};
use tsox_frontend::ast::{Node, Symbol, SymbolFlags, SyntaxKind};

static OPTIONAL_TYPE: OnceLock<Arc<Type>> = OnceLock::new();

pub(crate) fn optional_type_of(c: &mut Checker) -> Arc<Type> { ::tsox_core::fntrace::enter("optional_type_of"); 
    OPTIONAL_TYPE
        .get_or_init(|| c.new_intrinsic_type(TypeFlags::UNDEFINED, "undefined"))
        .clone()
}

impl Checker {
    pub(crate) fn get_regular_type_of_expression(&mut self, expr: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_regular_type_of_expression"); 
        let expr = if crate::checker::mig::m1a::is_right_side_of_access_expression(expr) {
            expr.parent().unwrap_or_else(|| Arc::clone(expr))
        } else {
            Arc::clone(expr)
        };
        let t = self.get_type_of_expression(&expr);
        self.get_regular_type_of_literal_type(&t)
    }

    pub(crate) fn get_binding_element_type_from_parent_type(
        &mut self,
        declaration: &Arc<Node>,
        parent_type: &Arc<Type>,
        no_tuple_bounds_check: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_binding_element_type_from_parent_type"); 
        if is_type_any(parent_type) {
            return Arc::clone(parent_type);
        }
        let pattern = declaration
            .parent()
            .unwrap_or_else(|| Arc::clone(declaration));
        let mut parent_type = Arc::clone(parent_type);
        let ambient = declaration.flags.contains(tsox_frontend::ast::NodeFlags::Ambient);
        if self.strict_null_checks
            && ambient
            && tsox_frontend::ast::mig::m3g_2::is_part_of_parameter_declaration(declaration)
        {
            parent_type = self.get_non_nullable_type(&parent_type);
        } else if self.strict_null_checks
            && pattern
                .parent()
                .as_ref()
                .and_then(|pp| tsox_frontend::ast::mig::m3b::initializer(pp))
                .is_some()
        {
            let init = pattern
                .parent()
                .as_ref()
                .and_then(|pp| tsox_frontend::ast::mig::m3b::initializer(pp))
                .cloned()
                .unwrap();
            let init_type = self.get_type_of_initializer(&init);
            if !self.has_type_facts(&init_type, TypeFacts::EQ_UNDEFINED) {
                parent_type = self.get_type_with_facts(&parent_type, TypeFacts::NE_UNDEFINED);
            }
        }
        let access_flags = AccessFlags::ExpressionPosition
            | if no_tuple_bounds_check || self.has_default_value(declaration) {
                AccessFlags::AllowMissing
            } else {
                AccessFlags::None
            };
        let t = match pattern.kind {
            SyntaxKind::ObjectBindingPattern => {
                if has_dot_dot_dot_token(declaration) {
                    parent_type = self.get_reduced_type(&parent_type);
                    if parent_type.flags.contains(TypeFlags::UNKNOWN)
                        || !self.is_valid_spread_type(&parent_type)
                    {
                        self.error_message(
                            declaration,tsox_core::diagnostics::messages_generated::REST_TYPES_MAY_ONLY_BE_CREATED_FROM_OBJECT_TYPES,
                            &[],
                        );
                        return self.error_type();
                    }
                    let elements = pattern
                        .elements()
                        .map(|l| l.nodes.clone())
                        .unwrap_or_default();
                    let mut literal_members: Vec<Arc<Node>> = Vec::new();
                    for element in &elements {
                        if !has_dot_dot_dot_token(element) {
                            if let Some(name) =
                                tsox_frontend::ast::mig::m3b::property_name_or_name(element)
                            {
                                literal_members.push(Arc::clone(name));
                            }
                        }
                    }
                    let sym = self.get_symbol_of_declaration(declaration);
                    self.get_rest_type(&parent_type, &literal_members, sym.as_ref())
                } else {
                    let name = tsox_frontend::ast::mig::m3b::property_name_or_name(declaration)
                        .map(Arc::clone)
                        .or_else(|| declaration.name().cloned())
                        .unwrap_or_else(|| Arc::clone(declaration));
                    let index_type = self
                        .get_literal_type_from_property_name(&name)
                        .unwrap_or_else(|| self.error_type());
                    let declared_type = self.get_indexed_access_type_ex(
                        &parent_type,
                        &index_type,
                        access_flags,
                        Some(&name),
                        None,
                    );
                    self.get_flow_type_of_destructuring(declaration, &declared_type)
                }
            }
            SyntaxKind::ArrayBindingPattern => {
                let element_type =
                    self.check_iterated_type_or_element_type(IterationUse::Destructuring, &parent_type, Some(&pattern));
                let index = pattern
                    .elements()
                    .map(|l| l.nodes.clone())
                    .unwrap_or_default()
                    .iter()
                    .position(|e| Arc::ptr_eq(e, declaration))
                    .unwrap_or(0);
                if has_dot_dot_dot_token(declaration) {
                    let self_ptr = self as *mut Checker;
                    let base_constraint = self.map_type(&parent_type, &mut |t: &Arc<Type>| {
                        if t.flags.contains(TypeFlags::INSTANTIABLE_NON_PRIMITIVE) {
                            Some(unsafe { &mut *self_ptr }.get_base_constraint_or_type(t))
                        } else {
                            Some(Arc::clone(t))
                        }
                    });
                    if base_constraint
                        .as_ref()
                        .is_some_and(|bc| every_type(bc, &is_tuple_type))
                    {
                        let base_constraint = base_constraint.unwrap();
                        self.map_type(&base_constraint, &mut |t: &Arc<Type>| {
                            unsafe { &mut *self_ptr }.slice_tuple_type(t, index, 0)
                        })
                        .unwrap_or_else(|| self.error_type())
                    } else {
                        self.create_array_type(element_type)
                    }
                } else if self.is_array_like_type(&parent_type) {
                    let number_type = self.number_type();
                    let index_type = self.get_number_literal_type(tsox_core::jsnum::Number::from(
                        index as f64,
                    ));
                    let indexed = self.get_indexed_access_type_or_undefined(
                        &parent_type,
                        &index_type,
                        access_flags,
                        declaration.name().map(|n| n.as_ref()),
                        None,
                    );
                    let declared_type = indexed.unwrap_or_else(|| self.error_type());
                    self.get_flow_type_of_destructuring(declaration, &declared_type)
                } else {
                    element_type
                }
            }
            _ => self.error_type(),
        };
        if tsox_frontend::ast::mig::m3b::initializer(declaration).is_none() {
            return t;
        }
        let walked = tsox_frontend::ast::mig::m3h::walk_up_binding_elements_and_patterns(declaration)
            .unwrap_or_else(|| Arc::clone(declaration));
        let walked_typed = !Arc::ptr_eq(&self.get_type_of_node(&walked), &self.error_type());
        if walked_typed {
            if self.strict_null_checks {
                let declared_init =
                    self.check_declaration_initializer(declaration, CheckMode::Normal, None);
                if !self.has_type_facts(&declared_init, TypeFacts::IS_UNDEFINED) {
                    return self.get_non_undefined_type(&t);
                }
            }
            return t;
        }
        let non_undefined = self.get_non_undefined_type(&t);
        let init_type = self.check_declaration_initializer(declaration, CheckMode::Normal, None);
        let union = self.get_union_type_ex(vec![non_undefined, init_type], UnionReduction::Subtype);
        self.widen_type_inferred_from_initializer(declaration, &union)
    }
}

fn has_dot_dot_dot_token(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_dot_dot_dot_token"); 
    matches!(
        &node.data,
        tsox_frontend::ast::NodeData::BindingElement(d) if d.dot_dot_dot_token.is_some()
    )
}

impl<'a> NodeBuilder<'a> {
    pub fn expand_symbol_for_hover_nodes(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("expand_symbol_for_hover_nodes"); 
        self.enter_context(
            None,
            NodeBuilderFlags::IGNORE_ERRORS
                | NodeBuilderFlags::MultilineObjectLiterals
                | NodeBuilderFlags::UseAliasDefinedOutsideCurrentScope,
            NodeBuilderInternalFlags::empty(),
            None,
        );
        let nodes = self.impl_.expand_symbol_for_hover(symbol);
        self.propagate_verbosity_out();
        let mut result: Vec<Arc<Node>> = Vec::new();
        for node in &nodes {
            match node.kind {
                SyntaxKind::ClassDeclaration => {
                    result.push(crate::checker::mig::m2f::simplify_class_declaration(
                        &self.impl_.f,
                        Arc::clone(node),
                        symbol,
                    ));
                }
                SyntaxKind::EnumDeclaration => {
                    result.push(crate::checker::mig::m2f::simplify_modifiers(
                        &self.impl_.f,
                        Arc::clone(node),
                        |n: &Arc<Node>| tsox_frontend::ast::is_enum_declaration(n),
                        symbol,
                    ));
                }
                SyntaxKind::InterfaceDeclaration => {
                    if meaning.intersects(SymbolFlags::Interface) {
                        result.push(crate::checker::mig::m2f::simplify_modifiers(
                            &self.impl_.f,
                            Arc::clone(node),
                            |n: &Arc<Node>| tsox_frontend::ast::is_interface_declaration(n),
                            symbol,
                        ));
                    }
                }
                SyntaxKind::ModuleDeclaration => {
                    result.push(crate::checker::mig::m2f::simplify_modifiers(
                        &self.impl_.f,
                        Arc::clone(node),
                        |n: &Arc<Node>| tsox_frontend::ast::is_module_declaration(n),
                        symbol,
                    ));
                }
                _ => {}
            }
        }
        self.exit_context_slice(result).unwrap_or_default()
    }

    pub fn signature_to_signature_declaration_ex(
        &mut self,
        signature: &Signature,
        kind: SyntaxKind,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("signature_to_signature_declaration_ex"); 
        let signature = Arc::new(signature.clone());
        self.enter_context(enclosing_declaration, flags, internal_flags, None);
        let result = self
            .impl_
            .signature_to_signature_declaration_helper(&signature, kind, None);
        self.exit_context(Some(result))
    }

    pub fn type_to_type_node_ex(
        &mut self,
        t: &Arc<Type>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("type_to_type_node_ex"); 
        self.enter_context(enclosing_declaration, flags, internal_flags, None);
        let result = self.impl_.type_to_type_node(t);
        self.exit_context(result)
    }

    pub fn type_predicate_to_type_predicate_node_ex(
        &mut self,
        predicate: &TypePredicate,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("type_predicate_to_type_predicate_node_ex"); 
        self.enter_context(enclosing_declaration, flags, internal_flags, None);
        let result = self.impl_.type_predicate_to_type_predicate_node(predicate);
        self.exit_context(Some(result))
    }
}
