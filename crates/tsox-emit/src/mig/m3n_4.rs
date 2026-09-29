use super::m3n::*;
use std::sync::Arc;
use super::m3n_5::create_diagnostic_for_node;
use tsox_checker::checker::types::SymbolAccessibilityResult;
use tsox_core::diagnostics::messages_generated as diag_msgs;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::diagnostic::Diagnostic;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{
    find_ancestor, is_assertion_expression, is_function_like_declaration, is_static, is_statement,
};
use tsox_frontend::scanner::mig::m3i::get_text_of_node;

pub fn get_type_parameter_constraint_visibility_diagnostic_message(
    node: &Arc<Node>,
    _symbol_accessibility_result: &SymbolAccessibilityResult,
) -> Option<&'static Message> {
    let Some(parent) = node.parent() else {
        panic!("This is unknown parent for type parameter: None");
    };
    match parent.kind {
        SyntaxKind::ClassDeclaration => Some(
            &diag_msgs::TYPE_PARAMETER_0_OF_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        SyntaxKind::InterfaceDeclaration => Some(
            &diag_msgs::TYPE_PARAMETER_0_OF_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        SyntaxKind::MappedType => Some(
            &diag_msgs::TYPE_PARAMETER_0_OF_EXPORTED_MAPPED_OBJECT_TYPE_IS_USING_PRIVATE_NAME_1,
        ),
        SyntaxKind::ConstructorType | SyntaxKind::ConstructSignature => Some(
            &diag_msgs::TYPE_PARAMETER_0_OF_CONSTRUCTOR_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        SyntaxKind::CallSignature => Some(
            &diag_msgs::TYPE_PARAMETER_0_OF_CALL_SIGNATURE_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => {
            if is_static(&parent) {
                Some(&diag_msgs::TYPE_PARAMETER_0_OF_PUBLIC_STATIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1)
            } else if matches!(parent.parent(), Some(gp) if gp.kind == SyntaxKind::ClassDeclaration) {
                Some(&diag_msgs::TYPE_PARAMETER_0_OF_PUBLIC_METHOD_FROM_EXPORTED_CLASS_HAS_OR_IS_USING_PRIVATE_NAME_1)
            } else {
                Some(&diag_msgs::TYPE_PARAMETER_0_OF_METHOD_FROM_EXPORTED_INTERFACE_HAS_OR_IS_USING_PRIVATE_NAME_1)
            }
        }
        SyntaxKind::FunctionType | SyntaxKind::FunctionDeclaration => Some(
            &diag_msgs::TYPE_PARAMETER_0_OF_EXPORTED_FUNCTION_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        SyntaxKind::InferType => Some(
            &diag_msgs::EXTENDS_CLAUSE_FOR_INFERRED_TYPE_0_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        SyntaxKind::TypeAliasDeclaration | SyntaxKind::JSTypeAliasDeclaration => Some(
            &diag_msgs::TYPE_PARAMETER_0_OF_EXPORTED_TYPE_ALIAS_HAS_OR_IS_USING_PRIVATE_NAME_1,
        ),
        _ => panic!("This is unknown parent for type parameter: {:?}", parent.kind),
    }
}

pub fn get_related_suggestion_by_declaration_kind(kind: SyntaxKind) -> Option<&'static Message> {
    match kind {
        SyntaxKind::ArrowFunction => Some(&diag_msgs::ADD_A_RETURN_TYPE_TO_THE_FUNCTION_EXPRESSION),
        SyntaxKind::FunctionExpression => Some(&diag_msgs::ADD_A_RETURN_TYPE_TO_THE_FUNCTION_EXPRESSION),
        SyntaxKind::MethodDeclaration => Some(&diag_msgs::ADD_A_RETURN_TYPE_TO_THE_METHOD),
        SyntaxKind::GetAccessor => Some(&diag_msgs::ADD_A_RETURN_TYPE_TO_THE_GET_ACCESSOR_DECLARATION),
        SyntaxKind::SetAccessor => Some(&diag_msgs::ADD_A_TYPE_TO_PARAMETER_OF_THE_SET_ACCESSOR_DECLARATION),
        SyntaxKind::FunctionDeclaration => Some(&diag_msgs::ADD_A_RETURN_TYPE_TO_THE_FUNCTION_DECLARATION),
        SyntaxKind::ConstructSignature => Some(&diag_msgs::ADD_A_RETURN_TYPE_TO_THE_FUNCTION_DECLARATION),
        SyntaxKind::Parameter => Some(&diag_msgs::ADD_A_TYPE_ANNOTATION_TO_THE_PARAMETER_0),
        SyntaxKind::VariableDeclaration => Some(&diag_msgs::ADD_A_TYPE_ANNOTATION_TO_THE_VARIABLE_0),
        SyntaxKind::PropertyDeclaration => Some(&diag_msgs::ADD_A_TYPE_ANNOTATION_TO_THE_PROPERTY_0),
        SyntaxKind::PropertySignature => Some(&diag_msgs::ADD_A_TYPE_ANNOTATION_TO_THE_PROPERTY_0),
        SyntaxKind::ExportAssignment => Some(&diag_msgs::MOVE_THE_EXPRESSION_IN_DEFAULT_EXPORT_TO_A_VARIABLE_AND_ADD_A_TYPE_ANNOTATION_TO_IT),
        _ => None,
    }
}

pub fn get_error_by_declaration_kind(kind: SyntaxKind) -> Option<&'static Message> {
    match kind {
        SyntaxKind::FunctionExpression => Some(&diag_msgs::FUNCTION_MUST_HAVE_AN_EXPLICIT_RETURN_TYPE_ANNOTATION_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::FunctionDeclaration => Some(&diag_msgs::FUNCTION_MUST_HAVE_AN_EXPLICIT_RETURN_TYPE_ANNOTATION_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::ArrowFunction => Some(&diag_msgs::FUNCTION_MUST_HAVE_AN_EXPLICIT_RETURN_TYPE_ANNOTATION_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::MethodDeclaration => Some(&diag_msgs::METHOD_MUST_HAVE_AN_EXPLICIT_RETURN_TYPE_ANNOTATION_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::ConstructSignature => Some(&diag_msgs::METHOD_MUST_HAVE_AN_EXPLICIT_RETURN_TYPE_ANNOTATION_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::GetAccessor => Some(&diag_msgs::AT_LEAST_ONE_ACCESSOR_MUST_HAVE_AN_EXPLICIT_TYPE_ANNOTATION_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::SetAccessor => Some(&diag_msgs::AT_LEAST_ONE_ACCESSOR_MUST_HAVE_AN_EXPLICIT_TYPE_ANNOTATION_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::Parameter => Some(&diag_msgs::PARAMETER_MUST_HAVE_AN_EXPLICIT_TYPE_ANNOTATION_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::VariableDeclaration => Some(&diag_msgs::VARIABLE_MUST_HAVE_AN_EXPLICIT_TYPE_ANNOTATION_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::PropertyDeclaration => Some(&diag_msgs::PROPERTY_MUST_HAVE_AN_EXPLICIT_TYPE_ANNOTATION_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::PropertySignature => Some(&diag_msgs::PROPERTY_MUST_HAVE_AN_EXPLICIT_TYPE_ANNOTATION_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::ComputedPropertyName => Some(&diag_msgs::COMPUTED_PROPERTY_NAMES_ON_CLASS_OR_OBJECT_LITERALS_CANNOT_BE_INFERRED_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::SpreadAssignment => Some(&diag_msgs::OBJECTS_THAT_CONTAIN_SPREAD_ASSIGNMENTS_CAN_T_BE_INFERRED_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::ShorthandPropertyAssignment => Some(&diag_msgs::OBJECTS_THAT_CONTAIN_SHORTHAND_PROPERTIES_CAN_T_BE_INFERRED_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::ArrayLiteralExpression => Some(&diag_msgs::ONLY_CONST_ARRAYS_CAN_BE_INFERRED_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::ExportAssignment => Some(&diag_msgs::DEFAULT_EXPORTS_CAN_T_BE_INFERRED_WITH_ISOLATEDDECLARATIONS),
        SyntaxKind::SpreadElement => Some(&diag_msgs::ARRAYS_WITH_SPREAD_ELEMENTS_CAN_T_INFERRED_WITH_ISOLATEDDECLARATIONS),
        _ => None,
    }
}

pub fn is_declaration_enough_for_errors(node: &Node) -> bool {
    is_export_assignment(node)
        || is_statement(node)
        || is_variable_declaration(node)
        || is_property_declaration(node)
        || is_parameter_declaration(node)
}

pub fn is_function_like_and_not_constructor(node: &Node) -> bool {
    is_function_like_declaration(node) && !is_constructor_declaration(node)
}

pub fn find_nearest_declaration(node: &Arc<Node>) -> Option<Arc<Node>> {
    let result = find_ancestor(node, is_declaration_enough_for_errors)?;
    if is_export_assignment(&result) {
        return Some(result);
    }
    if is_return_statement(&result) {
        return find_ancestor(&result, is_function_like_and_not_constructor);
    }
    if is_statement(&result) {
        return None;
    }
    Some(result)
}

pub enum FindAncestorResult {
    True,
    False,
    Quit,
}

pub fn to_find_ancestor_result(b: bool) -> FindAncestorResult {
    if b {
        FindAncestorResult::True
    } else {
        FindAncestorResult::False
    }
}

pub fn find_ancestor_or_quit<F>(node: &Arc<Node>, callback: F) -> Option<Arc<Node>>
where
    F: Fn(&Arc<Node>) -> FindAncestorResult,
{
    let mut current: Option<Arc<Node>> = Some(Arc::clone(node));
    while let Some(n) = current {
        match callback(&n) {
            FindAncestorResult::True => return Some(n),
            FindAncestorResult::Quit => return None,
            FindAncestorResult::False => {}
        }
        current = n.parent();
    }
    None
}

pub fn is_parent_for_id_diagnostic(node: &Arc<Node>) -> FindAncestorResult {
    if is_export_assignment(node) {
        return FindAncestorResult::True;
    }
    if is_statement(node) {
        return FindAncestorResult::Quit;
    }
    to_find_ancestor_result(!is_parenthesized_expression(node) && !is_assertion_expression(node))
}

pub fn create_object_literal_error(node: &Arc<Node>) -> Diagnostic {
    let mut diag = create_diagnostic_for_node(node, get_error_by_declaration_kind(node.kind), &[]);
    add_parent_declaration_related_info(node, &mut diag);
    diag
}

pub fn create_return_type_error(node: &Arc<Node>) -> Diagnostic {
    let mut diag = create_diagnostic_for_node(node, get_error_by_declaration_kind(node.kind), &[]);
    add_parent_declaration_related_info(node, &mut diag);
    diag.add_related_info(create_diagnostic_for_node(
        node,
        get_related_suggestion_by_declaration_kind(node.kind),
        &[],
    ));
    diag
}

pub fn create_variable_or_property_error(node: &Arc<Node>) -> Diagnostic {
    let mut diag = create_diagnostic_for_node(node, get_error_by_declaration_kind(node.kind), &[]);
    let name_arg = node
        .name()
        .map(|n| get_text_of_node(n))
        .unwrap_or_default();
    diag.add_related_info(create_diagnostic_for_node(
        node,
        get_related_suggestion_by_declaration_kind(node.kind),
        &[name_arg],
    ));
    diag
}

pub fn add_parent_declaration_related_info(node: &Arc<Node>, diag: &mut Diagnostic) {
    let Some(parent_declaration) = find_nearest_declaration(node) else {
        return;
    };
    let mut target_str = String::new();
    if !is_export_assignment(&parent_declaration) {
        if let Some(name) = parent_declaration.name() {
            target_str = get_text_of_node(name);
        }
    }
    diag.add_related_info(create_diagnostic_for_node(
        &parent_declaration,
        get_related_suggestion_by_declaration_kind(parent_declaration.kind),
        &[target_str],
    ));
}
