#![allow(unused_imports)]

use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::NodeFlags;
use tsox_frontend::ast::SyntaxKind;
use tsox_core::diagnostics::messages_generated::*;
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use tsox_frontend::ast::is_access_expression;
use tsox_frontend::ast::is_function_like;
use tsox_frontend::ast::mig::m3e_4::get_immediately_invoked_function_expression;
use tsox_frontend::ast::get_root_declaration;
use crate::checker::checker_ast_get_combined_modifier_flags::ast_get_combined_modifier_flags;
use super::{find_ancestor_node, is_global_source_file};

pub(crate) fn symbol_option_ptr_eq(a: &Option<Arc<Symbol>>, b: &Option<Arc<Symbol>>) -> bool { ::tsox_core::fntrace::enter("symbol_option_ptr_eq"); 
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}

impl Checker {
    pub fn check_module_export_name(&mut self, name: &Arc<Node>, allow_string_literal: bool) { ::tsox_core::fntrace::enter("check_module_export_name"); 
        if name.kind != SyntaxKind::StringLiteral {
            return;
        }
        if !allow_string_literal {
            self.grammar_error_on_node(name, &IDENTIFIER_EXPECTED);
        } else if self.module_kind == ModuleKind::ES2015 || self.module_kind == ModuleKind::ES2020 {
            let file = self.get_source_file_of_node(name);
            if file.as_ref().map(|f| !f.is_declaration_file).unwrap_or(false) {
                self.grammar_error_on_node(name, &STRING_LITERAL_IMPORT_AND_EXPORT_NAMES_ARE_NOT_SUPPORTED_WHEN_THE_MODULE_FLAG_IS_SET_TO_ES2015_OR_ES2020);
            }
        }
    }

    pub fn check_reference_expression(
        &mut self,
        expr: &Arc<Node>,
        invalid_reference_message: tsox_core::diagnostics::Message,
        invalid_optional_chain_message: tsox_core::diagnostics::Message,
    ) -> bool { ::tsox_core::fntrace::enter("check_reference_expression"); 
        let node = skip_outer_expressions(
            expr,
            OuterExpressionKinds::TYPE_ASSERTIONS | OuterExpressionKinds::SATISFIES | OuterExpressionKinds::PARENS,
        );
        if node.kind != SyntaxKind::Identifier && !is_access_expression(&node) {
            self.error_message(expr, invalid_reference_message, &[]);
            return false;
        }
        if node.flags.contains(NodeFlags::OptionalChain) {
            self.error_message(expr, invalid_optional_chain_message, &[]);
            return false;
        }
        true
    }

    pub fn get_control_flow_container(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_control_flow_container"); 
        let parent = node.parent().unwrap_or_else(|| Arc::clone(node));
        find_ancestor_node(&parent, |n| {
            (is_function_like(n) && get_immediately_invoked_function_expression(n).is_none())
                || tsox_frontend::ast::is_module_block(n)
                || n.kind == SyntaxKind::SourceFile
                || tsox_frontend::ast::is_property_declaration(n)
        })
        .unwrap_or(parent)
    }

    pub fn check_source_elements_from_list(&mut self, list: &[Arc<Node>]) { ::tsox_core::fntrace::enter("check_source_elements_from_list"); 
        for node in list {
            self.check_source_element(node);
        }
    }

    pub fn get_type_from_import_attributes_opt(&mut self, attributes: Option<&Arc<Node>>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_from_import_attributes_opt"); 
        match attributes {
            Some(attributes) => self.get_type_from_import_attributes(Some(attributes)),
            None => None,
        }
    }

    pub fn is_parameter_or_mutable_local_variable(&self, symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_parameter_or_mutable_local_variable"); 
        if let Some(value_declaration) = &symbol.value_declaration {
            let declaration = get_root_declaration(value_declaration);
            return declaration.kind == SyntaxKind::Parameter
                || (tsox_frontend::ast::is_variable_declaration(&declaration)
                    && (declaration.parent().map(|p| tsox_frontend::ast::is_catch_clause(&p)).unwrap_or(false)
                        || self.is_mutable_local_variable_declaration(&declaration)));
        }
        false
    }

    pub fn is_in_ambient_or_type_node(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_in_ambient_or_type_node"); 
        node.flags.contains(NodeFlags::Ambient)
            || self
                .get_source_file_of_node(node)
                .is_some_and(|f| f.is_declaration_file)
            || find_ancestor_node(node, |n| {
                n.kind == SyntaxKind::InterfaceDeclaration
                    || n.kind == SyntaxKind::TypeAliasDeclaration
                    || n.kind == SyntaxKind::TypeLiteral
            })
            .is_some()
    }
}
