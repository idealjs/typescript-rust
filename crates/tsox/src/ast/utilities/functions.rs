use crate::ast::*;

fn is_function_like_declaration_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
    )
}

pub fn is_function_like_declaration(node: &Node) -> bool {
    is_function_like_declaration_kind(node.kind)
}

pub fn is_function_like_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::MethodSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::JSDocSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
    ) || is_function_like_declaration_kind(kind)
}

pub fn is_function_like(node: &Node) -> bool {
    is_function_like_kind(node.kind)
}

pub fn is_function_like_or_class_static_block_declaration(node: &Node) -> bool {
    is_function_like(node) || is_class_static_block_declaration(node)
}

pub fn is_function_or_source_file(node: &Node) -> bool {
    is_function_like(node) || is_source_file(node)
}

pub fn is_class_like(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
    )
}

pub fn is_class_or_interface_like(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::InterfaceDeclaration
    )
}

pub fn is_class_element(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Constructor
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::IndexSignature
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::SemicolonClassElement
    )
}

pub fn is_method_or_accessor(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::MethodDeclaration | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
    )
}

pub fn is_type_element(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ConstructSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::NotEmittedTypeElement
    )
}

pub fn is_object_literal_element(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::SpreadAssignment
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
    )
}

pub fn is_accessor(node: &Node) -> bool {
    matches!(node.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor)
}

pub fn is_module_or_enum_declaration(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ModuleDeclaration | SyntaxKind::EnumDeclaration
    )
}

pub fn is_function_expression_or_arrow_function(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
    )
}
