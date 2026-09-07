#![allow(unused_imports)]

use super::*;

pub(crate) fn is_this_parameter(node: &Arc<Node>) -> bool {
    if node.kind != SyntaxKind::Parameter {
        return false;
    }
    match &node.data {
        NodeData::ParameterDeclaration(data) => {
            matches!(&data.name.data, NodeData::Identifier(id) if id.text == "this")
        }
        _ => false,
    }
}

pub(crate) fn is_variable_statement(node: &Arc<Node>) -> bool {
    node.kind == SyntaxKind::VariableStatement
}

pub(crate) fn is_parent_module_block_or_source_file(node: &Arc<Node>) -> bool {
    match &node.parent {
        Some(parent) => is_module_block(parent) || is_source_file(parent),
        None => false,
    }
}

pub(crate) fn is_parent_class_like(node: &Arc<Node>) -> bool {
    match &node.parent {
        Some(parent) => is_class_declaration(parent) || is_class_expression(parent),
        None => false,
    }
}

pub(crate) fn is_iteration_statement(node: &Arc<Node>, look_in_labeled: bool) -> bool {
    match node.kind {
        SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement
        | SyntaxKind::WhileStatement
        | SyntaxKind::DoStatement => true,
        SyntaxKind::LabeledStatement if look_in_labeled => {
            if let NodeData::LabeledStatement(data) = &node.data {
                is_iteration_statement(&data.statement, false)
            } else {
                false
            }
        }
        _ => false,
    }
}

pub(crate) fn is_function_like_or_class_static_block(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ClassStaticBlockDeclaration
    )
}

pub(crate) fn is_optional_declaration(node: &Arc<Node>) -> bool {
    if node.kind != SyntaxKind::Parameter {
        return false;
    }
    match &node.data {
        NodeData::ParameterDeclaration(data) => {
            data.question_token.is_some() || data.initializer.is_some()
        }
        _ => false,
    }
}

pub(crate) fn is_binding_pattern(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
    )
}

pub(crate) fn visibility_to_string(kind: SyntaxKind) -> &'static str {
    match kind {
        SyntaxKind::PublicKeyword => "public",
        SyntaxKind::ProtectedKeyword => "protected",
        SyntaxKind::PrivateKeyword => "private",
        _ => "",
    }
}

pub(crate) fn modifier_to_flag(kind: SyntaxKind) -> ModifierFlags {
    match kind {
        SyntaxKind::PublicKeyword => ModifierFlags::Public,
        SyntaxKind::ProtectedKeyword => ModifierFlags::Protected,
        SyntaxKind::PrivateKeyword => ModifierFlags::Private,
        SyntaxKind::StaticKeyword => ModifierFlags::Static,
        SyntaxKind::ReadonlyKeyword => ModifierFlags::Readonly,
        SyntaxKind::OverrideKeyword => ModifierFlags::Override,
        SyntaxKind::ExportKeyword => ModifierFlags::Export,
        SyntaxKind::AbstractKeyword => ModifierFlags::Abstract,
        SyntaxKind::DeclareKeyword => ModifierFlags::Ambient,
        SyntaxKind::AccessorKeyword => ModifierFlags::Accessor,
        SyntaxKind::AsyncKeyword => ModifierFlags::Async,
        SyntaxKind::DefaultKeyword => ModifierFlags::Default,
        SyntaxKind::ConstKeyword => ModifierFlags::Const,
        SyntaxKind::InKeyword => ModifierFlags::In,
        SyntaxKind::OutKeyword => ModifierFlags::Out,
        _ => ModifierFlags::empty(),
    }
}
