use crate::ast::*;
use std::sync::Arc;

use super::m3g_2::is_proto_setter;
use super::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use super::m3h::{climb_past_property_access, climb_past_property_or_element_access};
use super::w7a::is_external_module_indicator;

fn is_external_module_node(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_external_module_node"); 
    if !is_source_file(node) {
        return false;
    }
    match &node.data {
        NodeData::SourceFile(d) => d
            .statements
            .nodes
            .iter()
            .any(|stmt| is_external_module_indicator(stmt)),
        _ => false,
    }
}

pub fn is_jsx_call_like(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_jsx_call_like"); 
    matches!(
        node.kind,
        SyntaxKind::JsxOpeningElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxOpeningFragment
    )
}

pub fn is_jsx_opening_like_element(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_jsx_opening_like_element"); 
    is_jsx_opening_element(node) || is_jsx_self_closing_element(node)
}

fn select_expression_of_call_or_new_expression_or_decorator(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("select_expression_of_call_or_new_expression_or_decorator"); 
    match &node.data {
        NodeData::CallExpression(d) => Some(d.expression.clone()),
        NodeData::NewExpression(d) => Some(d.expression.clone()),
        NodeData::Decorator(d) => Some(d.expression.clone()),
        _ => None,
    }
}

fn select_tag_of_tagged_template_expression(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("select_tag_of_tagged_template_expression"); 
    match &node.data {
        NodeData::TaggedTemplateExpression(d) => Some(d.tag.clone()),
        _ => None,
    }
}

fn select_tag_name_of_jsx_opening_like_element(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("select_tag_name_of_jsx_opening_like_element"); 
    match &node.data {
        NodeData::JsxOpeningElement(d) => Some(d.tag_name.clone()),
        NodeData::JsxSelfClosingElement(d) => Some(d.tag_name.clone()),
        _ => None,
    }
}

fn is_callee_worker(
    node: &Arc<Node>,
    pred: &dyn Fn(&Node) -> bool,
    callee_selector: &dyn Fn(&Node) -> Option<Arc<Node>>,
    include_element_access: bool,
    skip_past_outer_expressions: bool,
) -> bool { ::tsox_core::fntrace::enter("is_callee_worker"); 
    let mut target: Arc<Node> = if include_element_access {
        climb_past_property_or_element_access(node)
    } else {
        climb_past_property_access(node)
    };
    if skip_past_outer_expressions && is_expression(&target) {
        target = skip_outer_expressions(&target, OuterExpressionKinds::ALL);
    }
    let Some(parent) = target.parent() else {
        return false;
    };
    pred(&parent) && callee_selector(&parent).is_some_and(|c| Arc::ptr_eq(&c, &target))
}

pub fn is_jsx_opening_like_element_tag_name(
    node: &Arc<Node>,
    include_element_access: bool,
    skip_past_outer_expressions: bool,
) -> bool { ::tsox_core::fntrace::enter("is_jsx_opening_like_element_tag_name"); 
    is_callee_worker(
        node,
        &is_jsx_opening_like_element,
        &select_tag_name_of_jsx_opening_like_element,
        include_element_access,
        skip_past_outer_expressions,
    )
}

pub fn is_new_expression_target(
    node: &Arc<Node>,
    include_element_access: bool,
    skip_past_outer_expressions: bool,
) -> bool { ::tsox_core::fntrace::enter("is_new_expression_target"); 
    is_callee_worker(
        node,
        &is_new_expression,
        &select_expression_of_call_or_new_expression_or_decorator,
        include_element_access,
        skip_past_outer_expressions,
    )
}

pub fn is_tagged_template_tag(
    node: &Arc<Node>,
    include_element_access: bool,
    skip_past_outer_expressions: bool,
) -> bool { ::tsox_core::fntrace::enter("is_tagged_template_tag"); 
    is_callee_worker(
        node,
        &is_tagged_template_expression,
        &select_tag_of_tagged_template_expression,
        include_element_access,
        skip_past_outer_expressions,
    )
}

pub fn is_jump_statement_target(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_jump_statement_target"); 
    if !is_identifier(node) {
        return false;
    }
    let Some(parent) = node.parent() else {
        return false;
    };
    if !is_break_or_continue_statement(&parent) {
        return false;
    }
    match &parent.data {
        NodeData::BreakStatement(d) => d.label.as_ref().is_some_and(|l| Arc::ptr_eq(l, node)),
        NodeData::ContinueStatement(d) => d.label.as_ref().is_some_and(|l| Arc::ptr_eq(l, node)),
        _ => false,
    }
}

pub fn is_label_name(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_label_name"); 
    is_label_of_labeled_statement(node) || is_jump_statement_target(node)
}

pub fn is_label_of_labeled_statement(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_label_of_labeled_statement"); 
    if !is_identifier(node) {
        return false;
    }
    let Some(parent) = node.parent() else {
        return false;
    };
    if !is_labeled_statement(&parent) {
        return false;
    }
    matches!(&parent.data, NodeData::LabeledStatement(d) if Arc::ptr_eq(&d.label, node))
}

pub fn is_late_visibility_painted_statement(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_late_visibility_painted_statement"); 
    matches!(
        node.kind,
        SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::VariableStatement
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::EnumDeclaration
    )
}

pub fn is_let(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_let"); 
    get_combined_node_flags(node).intersection(NodeFlags::BlockScoped) == NodeFlags::Let
}

pub fn is_literal_computed_property_declaration_name(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_literal_computed_property_declaration_name"); 
    if !is_string_or_numeric_literal_like(node) {
        return false;
    }
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind != SyntaxKind::ComputedPropertyName {
        return false;
    }
    match parent.parent() {
        Some(grand_parent) => is_declaration(&grand_parent),
        None => false,
    }
}

pub fn is_literal_like_element_access(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_literal_like_element_access"); 
    if !is_element_access_expression(node) {
        return false;
    }
    match &node.data {
        NodeData::ElementAccessExpression(d) => {
            is_string_or_numeric_literal_like(&d.argument_expression)
        }
        _ => false,
    }
}

pub fn is_logical_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_logical_expression"); 
    let mut node = node;
    loop {
        if node.kind == SyntaxKind::ParenthesizedExpression {
            match node.expression() {
                Some(next) => {
                    node = next;
                    continue;
                }
                None => return false,
            }
        }
        if node.kind == SyntaxKind::PrefixUnaryExpression {
            if let NodeData::PrefixUnaryExpression(d) = &node.data {
                if d.operator == SyntaxKind::ExclamationToken {
                    node = &d.operand;
                    continue;
                }
            }
        }
        return is_logical_or_coalescing_binary_expression(node);
    }
}

pub fn is_logical_or_coalescing_assignment_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_logical_or_coalescing_assignment_expression"); 
    match &node.data {
        NodeData::BinaryExpression(d) => {
            is_logical_or_coalescing_assignment_operator(d.operator_token.kind)
        }
        _ => false,
    }
}

pub fn is_logical_or_coalescing_binary_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_logical_or_coalescing_binary_expression"); 
    match &node.data {
        NodeData::BinaryExpression(d) => is_logical_or_coalescing_binary_operator(d.operator_token.kind),
        _ => false,
    }
}

pub fn is_modifier(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_modifier"); 
    is_modifier_kind(node.kind)
}

pub fn is_module_augmentation_external(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_module_augmentation_external"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind {
        SyntaxKind::SourceFile => is_external_module_node(&parent),
        SyntaxKind::ModuleBlock => {
            let Some(grand_parent) = parent.parent() else {
                return false;
            };
            let Some(great) = grand_parent.parent() else {
                return false;
            };
            is_ambient_module(&grand_parent)
                && great.kind == SyntaxKind::SourceFile
                && !is_external_module_node(&great)
        }
        _ => false,
    }
}

pub fn is_module_exports_access_expression(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_module_exports_access_expression"); 
    if is_access_expression(node) {
        if let Some(expression) = node.expression() {
            if is_module_identifier(expression) {
                let name: Option<&Arc<Node>> = match &node.data {
                    NodeData::PropertyAccessExpression(d) => {
                        (d.name.kind == SyntaxKind::Identifier).then_some(&d.name)
                    }
                    NodeData::ElementAccessExpression(d) => {
                        is_string_or_numeric_literal_like(&d.argument_expression)
                            .then_some(&d.argument_expression)
                    }
                    _ => None,
                };
                if let Some(name) = name {
                    return name.text() == "exports";
                }
            }
        }
    }
    false
}

pub fn is_module_exports_qualified_name(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_module_exports_qualified_name"); 
    match &node.data {
        NodeData::QualifiedName(d) => is_module_identifier(&d.left) && d.right.text() == "exports",
        _ => false,
    }
}

pub fn is_name_of_heritage_clause_type_reference(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_name_of_heritage_clause_type_reference"); 
    let mut current = Arc::clone(node);
    loop {
        let Some(parent) = current.parent() else {
            break;
        };
        if !is_qualified_name(&parent) {
            break;
        }
        current = parent;
    }
    let Some(parent) = current.parent() else {
        return false;
    };
    if !is_type_reference_node(&parent) {
        return false;
    }
    let NodeData::TypeReferenceNode(tr) = &parent.data else {
        return false;
    };
    if !Arc::ptr_eq(&tr.type_name, &current) {
        return false;
    }
    parent.parent().is_some_and(|gp| is_heritage_clause(&gp))
}

pub fn is_named_evaluation_source(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_named_evaluation_source"); 
    match &node.data {
        NodeData::PropertyAssignment(d) => !is_proto_setter(&d.name),
        NodeData::ShorthandPropertyAssignment(d) => d.object_assignment_initializer.is_some(),
        NodeData::VariableDeclaration(d) => is_identifier(&d.name) && d.initializer.is_some(),
        NodeData::ParameterDeclaration(d) => {
            is_identifier(&d.name) && d.initializer.is_some() && d.dot_dot_dot_token.is_none()
        }
        NodeData::BindingElement(d) => {
            d.name.as_ref().is_some_and(|n| is_identifier(n))
                && d.initializer.is_some()
                && d.dot_dot_dot_token.is_none()
        }
        NodeData::PropertyDeclaration(d) => d.initializer.is_some(),
        NodeData::BinaryExpression(d) => match d.operator_token.kind {
            SyntaxKind::EqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken => is_identifier(&d.left),
            _ => false,
        },
        NodeData::ExportAssignment(_) => true,
        _ => false,
    }
}
