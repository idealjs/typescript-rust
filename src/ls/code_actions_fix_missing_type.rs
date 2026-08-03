//! Fix missing type annotation (isolated declarations) code action
//! (1:1 port of Go's `internal/ls/codeactions_fixmissingtypeannotation.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::Node;
use crate::ast::SyntaxKind;
use crate::compiler::Program;
use crate::core::text::TextRange;

use super::code_actions::{CodeFixContext, CodeFixProvider};
use super::language_service::LanguageService;
use super::types::CodeAction;

/// Fix ID for missing type annotation.
pub const FIX_MISSING_TYPE_ANNOTATION_ON_EXPORTS_FIX_ID: &str = "fixMissingTypeAnnotationOnExports";

/// The `IsolatedDeclarationsFixProvider`.
pub fn isolated_declarations_fix_provider() -> CodeFixProvider {
    CodeFixProvider {
        error_codes: Vec::new(), // TODO: populate with diagnostic codes
        fix_ids: vec![FIX_MISSING_TYPE_ANNOTATION_ON_EXPORTS_FIX_ID.to_string()],
    }
}

impl LanguageService {
    /// Get isolated-declarations code actions.
    pub fn get_isolated_declarations_code_actions(
        &self,
        _context: &CodeFixContext,
    ) -> Vec<CodeAction> {
        // TODO: requires checker type inference + nodebuilder
        Vec::new()
    }

    /// Get all isolated-declarations code actions (fix-all).
    pub fn get_all_isolated_declarations_code_actions(
        &self,
        _context: &CodeFixContext,
    ) -> super::code_actions::CombinedCodeActions {
        // TODO: requires fix-all aggregation
        super::code_actions::CombinedCodeActions {
            description: String::new(),
            changes: Vec::new(),
        }
    }
}

/// Check if a node can have a type annotation added.
pub fn can_have_type_annotation(_node: &Arc<Node>) -> bool {
    use crate::ast::SyntaxKind;
    matches!(
        _node.kind,
        SyntaxKind::GetAccessor
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::ArrowFunction
            | SyntaxKind::FunctionExpression
            | SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::BindingElement
    )
}

/// Get the class declaration containing a span.
pub fn get_class(_file: &Arc<crate::ast::SourceFile>, _span: TextRange) -> Option<Arc<Node>> {
    // TODO: requires AST traversal
    None
}
