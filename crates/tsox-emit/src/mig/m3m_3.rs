use std::sync::Arc;

use tsox_core::diagnostics as diag;
use tsox_core::diagnostics::{Category, Message};

pub static BINDING_ELEMENTS_WITH_INITIALIZERS_CAN_T_BE_EXPORTED_DIRECTLY_WITH_ISOLATEDDECLARATIONS: Message =
    Message {
        code: 9019,
        category: Category::Error,
        key: "Binding_elements_with_initializers_can_t_be_exported_directly_with_isolatedDeclarations_9019",
        text: "Binding elements with initializers can't be exported directly with --isolatedDeclarations.",
        reports_unnecessary: false,
        elided_in_compatibility_pyramid: false,
        reports_deprecated: false,
    };
use tsox_frontend::ast;
use tsox_frontend::ast::Diagnostic;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SyntaxKind;
use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_frontend::ast::mig::m3e_4::{find_ancestor_or_quit, to_find_ancestor_result, FindAncestorResult};
use tsox_frontend::ast::mig::m3g_3::is_part_of_type_node;
use tsox_frontend::ast::mig::w3::get_all_accessor_declarations_for_declaration;
use tsox_frontend::scanner::mig::m3i::get_text_of_node;
use super::m3n_4::{
    add_parent_declaration_related_info, create_object_literal_error, create_return_type_error,
    create_variable_or_property_error, find_nearest_declaration, get_error_by_declaration_kind,
    get_related_suggestion_by_declaration_kind,
};
use super::m3n_5::create_diagnostic_for_node;

fn is_parent_for_idd_diagnostic_ast(node: &Arc<Node>) -> FindAncestorResult { ::tsox_core::fntrace::enter("is_parent_for_idd_diagnostic_ast"); 
    if ast::is_export_assignment(node) {
        return FindAncestorResult::True;
    }
    if ast::is_statement(node) {
        return FindAncestorResult::Quit;
    }
    to_find_ancestor_result(!ast::is_parenthesized_expression(node) && !ast::is_assertion_expression(node))
}

pub fn create_entity_in_type_node_error(node: &Arc<Node>) -> Diagnostic { ::tsox_core::fntrace::enter("create_entity_in_type_node_error"); 
    let text = get_text_of_node(node);
    let mut diag = create_diagnostic_for_node(
        node,
        Some(&diag::messages_generated::TYPE_CONTAINING_PRIVATE_NAME_0_CAN_T_BE_USED_WITH_ISOLATEDDECLARATIONS),
        &[text],
    );
    add_parent_declaration_related_info(node, &mut diag);
    diag
}

pub fn create_accessor_type_error(node: &Arc<Node>) -> Diagnostic { ::tsox_core::fntrace::enter("create_accessor_type_error"); 
    // 交接: Node 缺少 symbol() 访问器,Go 侧为 node.Symbol().Declarations,
    // 待 symbol map/resolver 接线后替换此行(见 progress_notes_r38k7.md);
    // 空声明切片仅影响 related-info 建议项,主诊断不受影响
    let all_declarations = get_all_accessor_declarations_for_declaration(node, &[]);
    let get_accessor = all_declarations.get_accessor;
    let set_accessor = all_declarations.set_accessor;
    let mut target_node = node;
    if ast::is_set_accessor_declaration(node) {
        if let Some(parameters) = node.parameters() {
            if let Some(first) = parameters.nodes.first() {
                target_node = first;
            }
        }
    }
    let mut diagnostic = create_diagnostic_for_node(
        target_node,
        get_error_by_declaration_kind(node.kind),
        &[],
    );
    if let Some(set_accessor) = set_accessor.as_ref() {
        diagnostic.add_related_info(create_diagnostic_for_node(
            set_accessor,
            get_related_suggestion_by_declaration_kind(set_accessor.kind),
            &[],
        ));
    }
    if let Some(get_accessor) = get_accessor.as_ref() {
        diagnostic.add_related_info(create_diagnostic_for_node(
            get_accessor,
            get_related_suggestion_by_declaration_kind(get_accessor.kind),
            &[],
        ));
    }
    diagnostic
}

pub fn create_array_literal_error(node: &Arc<Node>) -> Diagnostic { ::tsox_core::fntrace::enter("create_array_literal_error"); 
    let mut diagnostic =
        create_diagnostic_for_node(node, get_error_by_declaration_kind(node.kind), &[]);
    add_parent_declaration_related_info(node, &mut diagnostic);
    diagnostic
}

pub fn create_binding_element_error(node: &Arc<Node>) -> Diagnostic { ::tsox_core::fntrace::enter("create_binding_element_error"); 
    create_diagnostic_for_node(
        node,
        Some(&BINDING_ELEMENTS_WITH_INITIALIZERS_CAN_T_BE_EXPORTED_DIRECTLY_WITH_ISOLATEDDECLARATIONS),
        &[],
    )
}

pub fn create_expression_error(node: &Arc<Node>) -> Diagnostic { ::tsox_core::fntrace::enter("create_expression_error"); 
    create_expression_error_ex(node, None)
}

pub fn create_class_expression_error(node: &Arc<Node>) -> Diagnostic { ::tsox_core::fntrace::enter("create_class_expression_error"); 
    create_expression_error_ex(
        node,
        Some(&diag::messages_generated::INFERENCE_FROM_CLASS_EXPRESSIONS_IS_NOT_SUPPORTED_WITH_ISOLATEDDECLARATIONS),
    )
}

pub fn create_expression_error_ex(
    node: &Arc<Node>,
    diagnostic_message: Option<&'static Message>,
) -> Diagnostic { ::tsox_core::fntrace::enter("create_expression_error_ex"); 
    let Some(parent_declaration) = find_nearest_declaration(node) else {
        let diagnostic_message = diagnostic_message.unwrap_or(
            &diag::messages_generated::EXPRESSION_TYPE_CAN_T_BE_INFERRED_WITH_ISOLATEDDECLARATIONS,
        );
        return create_diagnostic_for_node(node, Some(diagnostic_message), &[]);
    };

    let mut target_str = String::new();
    if !ast::is_export_assignment(&parent_declaration) && parent_declaration.name().is_some() {
        target_str = get_text_of_node(parent_declaration.name().unwrap()).to_string();
    }
    let parent = find_ancestor_or_quit(node.parent().as_ref(), is_parent_for_idd_diagnostic_ast);

    let parent_matched = match &parent {
        Some(p) => Arc::ptr_eq(&parent_declaration, p),
        None => false,
    };
    if parent_matched {
        let diagnostic_message = diagnostic_message
            .unwrap_or(get_error_by_declaration_kind(parent_declaration.kind).unwrap_or(
                &diag::messages_generated::EXPRESSION_TYPE_CAN_T_BE_INFERRED_WITH_ISOLATEDDECLARATIONS,
            ));
        let mut diagnostic = create_diagnostic_for_node(node, Some(diagnostic_message), &[]);
        diagnostic.add_related_info(create_diagnostic_for_node(
            &parent_declaration,
            get_related_suggestion_by_declaration_kind(parent_declaration.kind),
            &[target_str],
        ));
        return diagnostic;
    }
    let diagnostic_message = diagnostic_message.unwrap_or(
        &diag::messages_generated::EXPRESSION_TYPE_CAN_T_BE_INFERRED_WITH_ISOLATEDDECLARATIONS,
    );
    let mut diagnostic = create_diagnostic_for_node(node, Some(diagnostic_message), &[]);
    diagnostic.add_related_info(create_diagnostic_for_node(
        &parent_declaration,
        get_related_suggestion_by_declaration_kind(parent_declaration.kind),
        &[target_str],
    ));
    diagnostic.add_related_info(create_diagnostic_for_node(
        node,
        Some(&diag::messages_generated::ADD_SATISFIES_AND_A_TYPE_ASSERTION_TO_THIS_EXPRESSION_SATISFIES_T_AS_T_TO_MAKE_THE_TYPE_EXPLICIT),
        &[],
    ));
    diagnostic
}

fn create_parameter_error(resolver: &EmitResolver, node: &Arc<Node>) -> Diagnostic { ::tsox_core::fntrace::enter("create_parameter_error"); 
    if let Some(parent) = node.parent() {
        if ast::is_set_accessor_declaration(&parent) {
            return create_accessor_type_error(&parent);
        }
    }
    let add_undefined = resolver.requires_adding_implicit_undefined_unsafe(node, None, None);
    if !add_undefined && node.initializer().is_some() {
        return create_expression_error(node.initializer().unwrap());
    }
    let mut message = get_error_by_declaration_kind(node.kind);
    if add_undefined {
        message = Some(
            &diag::messages_generated::DECLARATION_EMIT_FOR_THIS_PARAMETER_REQUIRES_IMPLICITLY_ADDING_UNDEFINED_TO_ITS_TYPE_THIS_IS_NOT_SUPPORTED_WITH_ISOLATEDDECLARATIONS,
        );
    }
    let mut diagnostic = create_diagnostic_for_node(node, message, &[]);
    let target_str = get_text_of_node(node.name().unwrap());
    diagnostic.add_related_info(create_diagnostic_for_node(
        node,
        get_related_suggestion_by_declaration_kind(node.kind),
        &[target_str],
    ));
    diagnostic
}

pub fn create_get_isolated_declaration_errors(
    resolver: EmitResolver,
) -> impl Fn(&Arc<Node>) -> Diagnostic { ::tsox_core::fntrace::enter("create_get_isolated_declaration_errors"); 
    move |node: &Arc<Node>| -> Diagnostic {
        let heritage_clause = ast::find_ancestor(node, ast::is_heritage_clause);
        if heritage_clause.is_some() {
            return create_diagnostic_for_node(
                node,
                Some(&diag::messages_generated::EXTENDS_CLAUSE_CAN_T_CONTAIN_AN_EXPRESSION_WITH_ISOLATEDDECLARATIONS),
                &[],
            );
        }
        if is_part_of_type_node(node) || ast::is_type_query_node(node) {
            return create_entity_in_type_node_error(node);
        }
        if ast::is_entity_name(node) || ast::is_entity_name_expression(node) {
            return create_entity_in_type_node_error(node);
        }
        match node.kind {
            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => create_accessor_type_error(node),
            SyntaxKind::ComputedPropertyName
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::SpreadAssignment => create_object_literal_error(node),
            SyntaxKind::ArrayLiteralExpression | SyntaxKind::SpreadElement => {
                create_array_literal_error(node)
            }
            SyntaxKind::MethodDeclaration
            | SyntaxKind::ConstructSignature
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::FunctionDeclaration => create_return_type_error(node),
            SyntaxKind::BindingElement => create_binding_element_error(node),
            SyntaxKind::PropertyDeclaration | SyntaxKind::VariableDeclaration => {
                create_variable_or_property_error(node)
            }
            SyntaxKind::Parameter => create_parameter_error(&resolver, node),
            SyntaxKind::PropertyAssignment => create_expression_error(node.initializer().unwrap()),
            SyntaxKind::ClassExpression => create_class_expression_error(node),
            _ => create_expression_error(node),
        }
    }
}
