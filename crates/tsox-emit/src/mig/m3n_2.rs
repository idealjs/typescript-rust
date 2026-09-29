use super::m3n::*;
use std::sync::Arc;
use tsox_checker::checker::types::{SymbolAccessibility, SymbolAccessibilityResult};
use tsox_core::diagnostics::messages_generated as diag_msgs;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{has_syntactic_modifier, is_static};

pub fn get_accessor_name_visibility_diagnostic_message(
    node: &Arc<Node>,
    symbol_accessibility_result: &SymbolAccessibilityResult,
) -> Option<&'static Message> {
    if is_static(node) {
        select_diagnostic_based_on_module_name(
            symbol_accessibility_result,
            &diag_msgs::PUBLIC_STATIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
            &diag_msgs::PUBLIC_STATIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::PUBLIC_STATIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
        )
    } else if parent_kind_is(node, SyntaxKind::ClassDeclaration) {
        select_diagnostic_based_on_module_name(
            symbol_accessibility_result,
            &diag_msgs::PUBLIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
            &diag_msgs::PUBLIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::PUBLIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
        )
    } else {
        select_diagnostic_based_on_module_name_no_name_check(
            symbol_accessibility_result,
            &diag_msgs::PROPERTY_0_OF_EXPORTED_INTERFACE_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::PROPERTY_0_OF_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_1,
        )
    }
}

pub fn get_method_name_visibility_diagnostic_message(
    node: &Arc<Node>,
    symbol_accessibility_result: &SymbolAccessibilityResult,
) -> Option<&'static Message> {
    if is_static(node) {
        select_diagnostic_based_on_module_name(
            symbol_accessibility_result,
            &diag_msgs::PUBLIC_STATIC_METHOD_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
            &diag_msgs::PUBLIC_STATIC_METHOD_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::PUBLIC_STATIC_METHOD_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
        )
    } else if parent_kind_is(node, SyntaxKind::ClassDeclaration) {
        select_diagnostic_based_on_module_name(
            symbol_accessibility_result,
            &diag_msgs::PUBLIC_METHOD_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
            &diag_msgs::PUBLIC_METHOD_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::PUBLIC_METHOD_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
        )
    } else {
        select_diagnostic_based_on_module_name_no_name_check(
            symbol_accessibility_result,
            &diag_msgs::METHOD_0_OF_EXPORTED_INTERFACE_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::METHOD_0_OF_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_1,
        )
    }
}

pub fn get_variable_declaration_type_visibility_diagnostic_message(
    node: &Arc<Node>,
    symbol_accessibility_result: &SymbolAccessibilityResult,
) -> Option<&'static Message> {
    if node.kind == SyntaxKind::VariableDeclaration || node.kind == SyntaxKind::BindingElement {
        return select_diagnostic_based_on_module_name(
            symbol_accessibility_result,
            &diag_msgs::EXPORTED_VARIABLE_0_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
            &diag_msgs::EXPORTED_VARIABLE_0_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::EXPORTED_VARIABLE_0_HAS_OR_IS_USING_PRIVATE_NAME_1,
        );
    }
    if node.kind == SyntaxKind::PropertyDeclaration
        || node.kind == SyntaxKind::PropertyAccessExpression
        || node.kind == SyntaxKind::ElementAccessExpression
        || node.kind == SyntaxKind::BinaryExpression
        || node.kind == SyntaxKind::PropertySignature
        || (node.kind == SyntaxKind::Parameter
            && node
                .parent()
                .map(|p| has_syntactic_modifier(&p, ModifierFlags::Private))
                .unwrap_or(false))
    {
        if is_static(node) {
            return select_diagnostic_based_on_module_name(
                symbol_accessibility_result,
                &diag_msgs::PUBLIC_STATIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
                &diag_msgs::PUBLIC_STATIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
                &diag_msgs::PUBLIC_STATIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
            );
        } else if parent_kind_is(node, SyntaxKind::ClassDeclaration) || node.kind == SyntaxKind::Parameter {
            return select_diagnostic_based_on_module_name(
                symbol_accessibility_result,
                &diag_msgs::PUBLIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
                &diag_msgs::PUBLIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
                &diag_msgs::PUBLIC_PROPERTY_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
            );
        } else {
            return select_diagnostic_based_on_module_name_no_name_check(
                symbol_accessibility_result,
                &diag_msgs::PROPERTY_0_OF_EXPORTED_INTERFACE_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
                &diag_msgs::PROPERTY_0_OF_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_1,
            );
        }
    }
    None
}

pub fn get_accessor_declaration_type_visibility_diagnostic_message(
    node: &Arc<Node>,
    symbol_accessibility_result: &SymbolAccessibilityResult,
) -> Option<&'static Message> {
    if node.kind == SyntaxKind::SetAccessor {
        if is_static(node) {
            select_diagnostic_based_on_module_name_no_name_check(
                symbol_accessibility_result,
                &diag_msgs::PARAMETER_TYPE_OF_PUBLIC_STATIC_SETTER_0_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
                &diag_msgs::PARAMETER_TYPE_OF_PUBLIC_STATIC_SETTER_0_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
            )
        } else {
            select_diagnostic_based_on_module_name_no_name_check(
                symbol_accessibility_result,
                &diag_msgs::PARAMETER_TYPE_OF_PUBLIC_SETTER_0_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
                &diag_msgs::PARAMETER_TYPE_OF_PUBLIC_SETTER_0_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
            )
        }
    } else if is_static(node) {
        select_diagnostic_based_on_module_name(
            symbol_accessibility_result,
            &diag_msgs::RETURN_TYPE_OF_PUBLIC_STATIC_GETTER_0_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
            &diag_msgs::RETURN_TYPE_OF_PUBLIC_STATIC_GETTER_0_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::RETURN_TYPE_OF_PUBLIC_STATIC_GETTER_0_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
        )
    } else {
        select_diagnostic_based_on_module_name(
            symbol_accessibility_result,
            &diag_msgs::RETURN_TYPE_OF_PUBLIC_GETTER_0_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
            &diag_msgs::RETURN_TYPE_OF_PUBLIC_GETTER_0_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::RETURN_TYPE_OF_PUBLIC_GETTER_0_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
        )
    }
}
