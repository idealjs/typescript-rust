use super::m3n::*;
use std::sync::Arc;
use tsox_checker::checker::types::SymbolAccessibilityResult;
use tsox_core::diagnostics::messages_generated as diag_msgs;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::is_static;

pub fn get_return_type_visibility_diagnostic_message(
    node: &Arc<Node>,
    symbol_accessibility_result: &SymbolAccessibilityResult,
) -> Option<&'static Message> { ::tsox_core::fntrace::enter("get_return_type_visibility_diagnostic_message"); 
    match node.kind {
        SyntaxKind::ConstructSignature => select_diagnostic_based_on_module_name_no_name_check(
            symbol_accessibility_result,
            &diag_msgs::RETURN_TYPE_OF_CONSTRUCTOR_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_NAME_0_FROM_PRIVATE_MODULE_1,
            &diag_msgs::RETURN_TYPE_OF_CONSTRUCTOR_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_0,
        ),
        SyntaxKind::CallSignature => select_diagnostic_based_on_module_name_no_name_check(
            symbol_accessibility_result,
            &diag_msgs::RETURN_TYPE_OF_CALL_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_NAME_0_FROM_PRIVATE_MODULE_1,
            &diag_msgs::RETURN_TYPE_OF_CALL_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_0,
        ),
        SyntaxKind::IndexSignature => select_diagnostic_based_on_module_name_no_name_check(
            symbol_accessibility_result,
            &diag_msgs::RETURN_TYPE_OF_INDEX_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_NAME_0_FROM_PRIVATE_MODULE_1,
            &diag_msgs::RETURN_TYPE_OF_INDEX_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_0,
        ),
        SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => {
            if is_static(node) {
                select_diagnostic_based_on_module_name(
                    symbol_accessibility_result,
                    &diag_msgs::RETURN_TYPE_OF_PUBLIC_STATIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_0_FROM_EXTERNAL_MODULE_1_BUT_CANNOT_BE_NAMED,
                    &diag_msgs::RETURN_TYPE_OF_PUBLIC_STATIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_0_FROM_PRIVATE_MODULE_1,
                    &diag_msgs::RETURN_TYPE_OF_PUBLIC_STATIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_0,
                )
            } else if parent_kind_is(node, SyntaxKind::ClassDeclaration) {
                select_diagnostic_based_on_module_name(
                    symbol_accessibility_result,
                    &diag_msgs::RETURN_TYPE_OF_PUBLIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_0_FROM_EXTERNAL_MODULE_1_BUT_CANNOT_BE_NAMED,
                    &diag_msgs::RETURN_TYPE_OF_PUBLIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_0_FROM_PRIVATE_MODULE_1,
                    &diag_msgs::RETURN_TYPE_OF_PUBLIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_0,
                )
            } else {
                select_diagnostic_based_on_module_name_no_name_check(
                    symbol_accessibility_result,
                    &diag_msgs::RETURN_TYPE_OF_METHOD_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_NAME_0_FROM_PRIVATE_MODULE_1,
                    &diag_msgs::RETURN_TYPE_OF_METHOD_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_0,
                )
            }
        }
        SyntaxKind::FunctionDeclaration => select_diagnostic_based_on_module_name(
            symbol_accessibility_result,
            &diag_msgs::RETURN_TYPE_OF_EXPORTED_FUNCTION_HAS_OR_IS_USING_NAME_0_FROM_EXTERNAL_MODULE_1_BUT_CANNOT_BE_NAMED,
            &diag_msgs::RETURN_TYPE_OF_EXPORTED_FUNCTION_HAS_OR_IS_USING_NAME_0_FROM_PRIVATE_MODULE_1,
            &diag_msgs::RETURN_TYPE_OF_EXPORTED_FUNCTION_HAS_OR_IS_USING_PRIVATE_NAME_0,
        ),
        _ => panic!(
            "This is unknown kind for signature: {:?}",
            node.kind
        ),
    }
}

pub fn get_parameter_declaration_type_visibility_diagnostic_message(
    node: &Arc<Node>,
    symbol_accessibility_result: &SymbolAccessibilityResult,
) -> Option<&'static Message> { ::tsox_core::fntrace::enter("get_parameter_declaration_type_visibility_diagnostic_message"); 
    let Some(parent) = node.parent() else {
        panic!("Unknown parent for parameter: None");
    };
    match parent.kind {
        SyntaxKind::Constructor => select_diagnostic_based_on_module_name(
            symbol_accessibility_result,
            &diag_msgs::PARAMETER_0_OF_CONSTRUCTOR_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
            &diag_msgs::PARAMETER_0_OF_CONSTRUCTOR_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::PARAMETER_0_OF_CONSTRUCTOR_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        SyntaxKind::ConstructSignature | SyntaxKind::ConstructorType => {
            select_diagnostic_based_on_module_name_no_name_check(
                symbol_accessibility_result,
                &diag_msgs::PARAMETER_0_OF_CONSTRUCTOR_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
                &diag_msgs::PARAMETER_0_OF_CONSTRUCTOR_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_1,
            )
        }
        SyntaxKind::CallSignature => select_diagnostic_based_on_module_name_no_name_check(
            symbol_accessibility_result,
            &diag_msgs::PARAMETER_0_OF_CALL_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::PARAMETER_0_OF_CALL_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        SyntaxKind::IndexSignature => select_diagnostic_based_on_module_name_no_name_check(
            symbol_accessibility_result,
            &diag_msgs::PARAMETER_0_OF_INDEX_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::PARAMETER_0_OF_INDEX_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => {
            if is_static(&parent) {
                select_diagnostic_based_on_module_name(
                    symbol_accessibility_result,
                    &diag_msgs::PARAMETER_0_OF_PUBLIC_STATIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
                    &diag_msgs::PARAMETER_0_OF_PUBLIC_STATIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
                    &diag_msgs::PARAMETER_0_OF_PUBLIC_STATIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
                )
            } else if matches!(parent.parent(), Some(gp) if gp.kind == SyntaxKind::ClassDeclaration) {
                select_diagnostic_based_on_module_name(
                    symbol_accessibility_result,
                    &diag_msgs::PARAMETER_0_OF_PUBLIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
                    &diag_msgs::PARAMETER_0_OF_PUBLIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
                    &diag_msgs::PARAMETER_0_OF_PUBLIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
                )
            } else {
                select_diagnostic_based_on_module_name_no_name_check(
                    symbol_accessibility_result,
                    &diag_msgs::PARAMETER_0_OF_METHOD_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
                    &diag_msgs::PARAMETER_0_OF_METHOD_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_1,
                )
            }
        }
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionType
        | SyntaxKind::ArrowFunction
        | SyntaxKind::FunctionExpression => select_diagnostic_based_on_module_name(
            symbol_accessibility_result,
            &diag_msgs::PARAMETER_0_OF_EXPORTED_FUNCTION_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
            &diag_msgs::PARAMETER_0_OF_EXPORTED_FUNCTION_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::PARAMETER_0_OF_EXPORTED_FUNCTION_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        SyntaxKind::SetAccessor | SyntaxKind::GetAccessor => select_diagnostic_based_on_module_name(
            symbol_accessibility_result,
            &diag_msgs::PARAMETER_0_OF_ACCESSOR_HAS_OR_IS_USING_NAME_1_FROM_EXTERNAL_MODULE_2_BUT_CANNOT_BE_NAMED,
            &diag_msgs::PARAMETER_0_OF_ACCESSOR_HAS_OR_IS_USING_NAME_1_FROM_PRIVATE_MODULE_2,
            &diag_msgs::PARAMETER_0_OF_ACCESSOR_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        _ => panic!("Unknown parent for parameter: {:?}", parent.kind),
    }
}
