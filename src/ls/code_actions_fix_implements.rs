#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::Node;
use crate::checker::Checker;

use super::code_actions::{CodeFixContext, CodeFixProvider};
use super::language_service::LanguageService;
use super::types::CodeAction;

pub const FIX_CLASS_INCORRECTLY_IMPLEMENTS_INTERFACE_FIX_ID: &str =
    "fixClassIncorrectlyImplementsInterface";

pub fn fix_class_incorrectly_implements_interface_provider() -> CodeFixProvider {
    CodeFixProvider {
        error_codes: Vec::new(),
        fix_ids: vec![FIX_CLASS_INCORRECTLY_IMPLEMENTS_INTERFACE_FIX_ID.to_string()],
    }
}

impl LanguageService {

    pub fn get_code_actions_to_fix_class_incorrectly_implements_interface(
        &self,
        _context: &CodeFixContext,
    ) -> Vec<CodeAction> {

        Vec::new()
    }

    pub fn get_all_code_actions_to_fix_class_incorrectly_implements_interface(
        &self,
        _context: &CodeFixContext,
    ) -> super::code_actions::CombinedCodeActions {

        super::code_actions::CombinedCodeActions {
            description: String::new(),
            changes: Vec::new(),
        }
    }
}

pub fn add_changes(
    _change_tracker: &crate::ls::change::Tracker,
    _type_checker: &Checker,
    _class_declaration: &Arc<Node>,
    _implemented_type_node: &Arc<Node>,
) {

}

pub fn get_changes(
    _change_tracker: &crate::ls::change::Tracker,
    _file: &Arc<crate::ast::SourceFile>,
) -> Vec<crate::lsp::lsproto::lsp::TextEdit> {

    Vec::new()
}

pub fn create_import_adder(
    _context: &CodeFixContext,
    _type_checker: &Checker,
) -> Result<(), String> {

    Ok(())
}
