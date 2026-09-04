#![allow(dead_code)]

use super::code_actions::{CodeFixContext, CodeFixProvider};
use super::language_service::LanguageService;
use super::types::CodeAction;

pub const IMPORT_FIX_ID: &str = "fixMissingImport";

pub fn import_fix_provider() -> CodeFixProvider {
    CodeFixProvider {
        error_codes: Vec::new(),
        fix_ids: vec![IMPORT_FIX_ID.to_string()],
    }
}

pub struct FixInfo {
    pub symbol_name: String,
    pub error_identifier_text: String,
}

impl LanguageService {

    pub fn get_import_code_actions(&self, _context: &CodeFixContext) -> Vec<CodeAction> {

        Vec::new()
    }

    pub fn get_all_import_code_actions(
        &self,
        _context: &CodeFixContext,
    ) -> super::code_actions::CombinedCodeActions {

        super::code_actions::CombinedCodeActions {
            description: String::new(),
            changes: Vec::new(),
        }
    }
}

pub fn get_symbol_name_from_error_text(text: &str) -> &str {
    text
}
