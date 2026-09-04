use std::sync::Arc;

use crate::ast::{Node, SourceFile, SyntaxKind};

pub fn position_is_asi_candidate(
    _pos: usize,
    _context: Option<&Arc<Node>>,
    _file: &SourceFile,
) -> bool {

    false
}

pub fn syntax_may_be_asi_candidate(kind: SyntaxKind) -> bool {
    syntax_requires_trailing_comma_or_semicolon_or_asi(kind)
        || syntax_requires_trailing_function_block_or_semicolon_or_asi(kind)
        || syntax_requires_trailing_module_block_or_semicolon_or_asi(kind)
        || syntax_requires_trailing_semicolon_or_asi(kind)
}

pub fn syntax_requires_trailing_comma_or_semicolon_or_asi(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodSignature
    )
}

pub fn syntax_requires_trailing_function_block_or_semicolon_or_asi(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
    )
}

pub fn syntax_requires_trailing_module_block_or_semicolon_or_asi(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::ModuleDeclaration)
}

pub fn syntax_requires_trailing_semicolon_or_asi(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::VariableStatement
            | SyntaxKind::ExpressionStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::ContinueStatement
            | SyntaxKind::BreakStatement
            | SyntaxKind::ReturnStatement
            | SyntaxKind::ThrowStatement
            | SyntaxKind::DebuggerStatement
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::ExportAssignment
    )
}

pub fn node_is_asi_candidate(_node: &Arc<Node>, _file: &SourceFile) -> bool {

    false
}
