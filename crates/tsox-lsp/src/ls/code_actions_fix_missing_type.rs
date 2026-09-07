#![allow(dead_code)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::Node;

use super::code_actions::{CodeFixContext, CodeFixProvider};
use super::language_service::LanguageService;
use super::types::CodeAction;

pub const FIX_MISSING_TYPE_ANNOTATION_ON_EXPORTS_FIX_ID: &str = "fixMissingTypeAnnotationOnExports";

pub fn isolated_declarations_fix_provider() -> CodeFixProvider {
    CodeFixProvider {
        error_codes: Vec::new(),
        fix_ids: vec![FIX_MISSING_TYPE_ANNOTATION_ON_EXPORTS_FIX_ID.to_string()],
    }
}

impl LanguageService {
    pub fn get_isolated_declarations_code_actions(
        &self,
        _context: &CodeFixContext,
    ) -> Vec<CodeAction> {
        Vec::new()
    }

    pub fn get_all_isolated_declarations_code_actions(
        &self,
        _context: &CodeFixContext,
    ) -> super::code_actions::CombinedCodeActions {
        super::code_actions::CombinedCodeActions {
            description: String::new(),
            changes: Vec::new(),
        }
    }
}

pub fn can_have_type_annotation(_node: &Arc<Node>) -> bool {
    use tsox_frontend::ast::SyntaxKind;
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

pub fn get_class(
    _file: &Arc<tsox_frontend::ast::SourceFile>,
    _span: TextRange,
) -> Option<Arc<Node>> {
    None
}
