//! Automatic semicolon insertion (ASI) helpers.
//!
//! Ported from `internal/ls/lsutil/asi.go`. The `Syntax*` predicate functions
//! are pure `SyntaxKind` checks and are ported in full. `PositionIsASICandidate`
//! and `NodeIsASICandidate` depend on AST traversal and scanner helpers that
//! are not yet ported, so their bodies are stubbed.

use std::sync::Arc;

use crate::ast::{Node, SourceFile, SyntaxKind};

/// Returns whether `pos` is a candidate for automatic semicolon insertion in
/// `context`.
///
/// Mirrors `PositionIsASICandidate` in Go.
pub fn position_is_asi_candidate(
    _pos: usize,
    _context: Option<&Arc<Node>>,
    _file: &SourceFile,
) -> bool {
    // TODO: requires ast.FindAncestorOrQuit + NodeIsASICandidate.
    false
}

/// Whether a syntax kind may be subject to ASI at all.
///
/// Mirrors `SyntaxMayBeASICandidate` in Go.
pub fn syntax_may_be_asi_candidate(kind: SyntaxKind) -> bool {
    syntax_requires_trailing_comma_or_semicolon_or_asi(kind)
        || syntax_requires_trailing_function_block_or_semicolon_or_asi(kind)
        || syntax_requires_trailing_module_block_or_semicolon_or_asi(kind)
        || syntax_requires_trailing_semicolon_or_asi(kind)
}

/// Mirrors `SyntaxRequiresTrailingCommaOrSemicolonOrASI` in Go.
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

/// Mirrors `SyntaxRequiresTrailingFunctionBlockOrSemicolonOrASI` in Go.
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

/// Mirrors `SyntaxRequiresTrailingModuleBlockOrSemicolonOrASI` in Go.
pub fn syntax_requires_trailing_module_block_or_semicolon_or_asi(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::ModuleDeclaration)
}

/// Mirrors `SyntaxRequiresTrailingSemicolonOrASI` in Go.
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

/// Whether a node is an ASI candidate.
///
/// Mirrors `NodeIsASICandidate` in Go. Depends on [`get_last_token`] and
/// scanner line-position helpers; stubbed until those are ported.
pub fn node_is_asi_candidate(_node: &Arc<Node>, _file: &SourceFile) -> bool {
    // TODO: requires GetLastToken + scanner.GetECMALineOfPosition + astnav.FindNextToken.
    false
}
