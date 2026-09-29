use std::sync::Arc;

use tsox_checker::checker::types::SymbolAccessibilityResult;
use tsox_core::diagnostics as diag;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::mig::m3g_2::is_parameter_property_declaration;

use super::m3n::{
    select_diagnostic_based_on_module_name, select_diagnostic_based_on_module_name_no_name_check,
    wrap_fallback_error_diagnostic_selector, wrap_named_diagnostic_selector,
    wrap_simple_diagnostic_selector, GetSymbolAccessibilityDiagnostic,
    SymbolAccessibilityDiagnostic,
};
use super::m3n_2::{
    get_accessor_declaration_type_visibility_diagnostic_message,
    get_variable_declaration_type_visibility_diagnostic_message,
};
use super::m3n_3::{
    get_parameter_declaration_type_visibility_diagnostic_message,
    get_return_type_visibility_diagnostic_message,
};
use super::m3n_4::get_type_parameter_constraint_visibility_diagnostic_message;

pub fn create_get_symbol_accessibility_diagnostic_for_node(
    node: &Arc<Node>,
) -> GetSymbolAccessibilityDiagnostic {
    if ast::is_variable_declaration(node)
        || ast::is_property_declaration(node)
        || ast::is_property_signature_declaration(node)
        || ast::is_property_access_expression(node)
        || ast::is_element_access_expression(node)
        || ast::is_binary_expression(node)
        || ast::is_binding_element(node)
        || ast::is_constructor_declaration(node)
    {
        wrap_simple_diagnostic_selector(
            node,
            get_variable_declaration_type_visibility_diagnostic_message,
        )
    } else if ast::is_set_accessor_declaration(node) || ast::is_get_accessor_declaration(node) {
        wrap_named_diagnostic_selector(
            node,
            get_accessor_declaration_type_visibility_diagnostic_message,
        )
    } else if ast::is_construct_signature_declaration(node)
        || ast::is_call_signature_declaration(node)
        || ast::is_method_declaration(node)
        || ast::is_method_signature_declaration(node)
        || ast::is_function_declaration(node)
        || ast::is_index_signature_declaration(node)
    {
        wrap_fallback_error_diagnostic_selector(node, get_return_type_visibility_diagnostic_message)
    } else if ast::is_parameter_declaration(node) {
        let parent = node.parent();
        if parent
            .as_deref()
            .is_some_and(|parent| {
                is_parameter_property_declaration(node, parent)
                    && ast::has_syntactic_modifier(parent, ast::ModifierFlags::Private)
            })
        {
            wrap_simple_diagnostic_selector(
                node,
                get_variable_declaration_type_visibility_diagnostic_message,
            )
        } else {
            wrap_simple_diagnostic_selector(
                node,
                get_parameter_declaration_type_visibility_diagnostic_message,
            )
        }
    } else if ast::is_type_parameter_declaration(node) {
        wrap_simple_diagnostic_selector(
            node,
            get_type_parameter_constraint_visibility_diagnostic_message,
        )
    } else if ast::is_expression_with_type_arguments(node) {
        let node = Arc::clone(node);
        Box::new(move |_symbol_accessibility_result: &SymbolAccessibilityResult| {
            let parent = node.parent();
            let grandparent = parent.as_ref().and_then(|p| p.parent());
            let diagnostic_message: &'static Message =
                if grandparent.as_ref().is_some_and(|g| ast::is_class_declaration(g)) {
                    if parent.as_ref().is_some_and(|p| {
                        ast::is_heritage_clause(p)
                            && matches!(
                                &p.data,
                                NodeData::HeritageClause(d)
                                    if d.token == SyntaxKind::ImplementsKeyword
                            )
                    }) {
                        &diag::messages_generated::IMPLEMENTS_CLAUSE_OF_EXPORTED_CLASS_0_HAS_OR_IS_USING_PRIVATE_NAME_1
                    } else if grandparent
                        .as_ref()
                        .is_some_and(|g| g.name().is_some())
                    {
                        &diag::messages_generated::X_EXTENDS_CLAUSE_OF_EXPORTED_CLASS_0_HAS_OR_IS_USING_PRIVATE_NAME_1
                    } else {
                        &diag::messages_generated::X_EXTENDS_CLAUSE_OF_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_0
                    }
                } else {
                    &diag::messages_generated::X_EXTENDS_CLAUSE_OF_EXPORTED_INTERFACE_0_HAS_OR_IS_USING_PRIVATE_NAME_1
                };
            Some(SymbolAccessibilityDiagnostic {
                diagnostic_message,
                error_node: Some(Arc::clone(&node)),
                type_name: grandparent
                    .as_ref()
                    .and_then(ast::get_name_of_declaration),
            })
        })
    } else if ast::is_import_equals_declaration(node) {
        wrap_simple_diagnostic_selector(node, |_node, _result| {
            Some(&diag::messages_generated::IMPORT_DECLARATION_0_IS_USING_PRIVATE_NAME_1)
        })
    } else if ast::is_type_alias_declaration(node) || ast::is_js_type_alias_declaration(node) {
        let node = Arc::clone(node);
        Box::new(move |symbol_accessibility_result: &SymbolAccessibilityResult| {
            let diagnostic_message = select_diagnostic_based_on_module_name_no_name_check(
                symbol_accessibility_result,
                &diag::messages_generated::EXPORTED_TYPE_ALIAS_0_HAS_OR_IS_USING_PRIVATE_NAME_1_FROM_MODULE_2,
                &diag::messages_generated::EXPORTED_TYPE_ALIAS_0_HAS_OR_IS_USING_PRIVATE_NAME_1,
            )
            .expect("type alias selector always yields a message");
            let error_node = match &node.data {
                NodeData::TypeAliasDeclaration(d) => Some(d.type_node.clone()),
                _ => None,
            };
            let type_name = node.name().cloned();
            Some(SymbolAccessibilityDiagnostic {
                error_node,
                diagnostic_message,
                type_name,
            })
        })
    } else if ast::is_call_expression(node) {
        let node = Arc::clone(node);
        Box::new(move |symbol_accessibility_result: &SymbolAccessibilityResult| {
            let diagnostic_message = select_diagnostic_based_on_module_name(
                symbol_accessibility_result,
                &diag::messages_generated::EXPORTED_VARIABLE_0_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
                &diag::messages_generated::EXPORTED_VARIABLE_0_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
                &diag::messages_generated::EXPORTED_VARIABLE_0_HAS_OR_IS_USING_PRIVATE_NAME_1,
            )
            .expect("call expression selector always yields a message");
            let arg = node.arguments().and_then(|l| l.nodes.get(1)).cloned();
            Some(SymbolAccessibilityDiagnostic {
                error_node: arg.clone(),
                diagnostic_message,
                type_name: arg,
            })
        })
    } else {
        panic!(
            "Attempted to set a declaration diagnostic context for unhandled node kind: {:?}",
            node.kind
        )
    }
}
