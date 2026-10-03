#![allow(dead_code)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_frontend::ast::Node;

use super::code_actions::{CodeFixContext, CodeFixProvider};
use super::language_service::LanguageService;
use super::types::CodeAction;

pub const FIX_CLASS_INCORRECTLY_IMPLEMENTS_INTERFACE_FIX_ID: &str =
    "fixClassIncorrectlyImplementsInterface";

pub fn fix_class_incorrectly_implements_interface_provider() -> CodeFixProvider { ::tsox_core::fntrace::enter("fix_class_incorrectly_implements_interface_provider"); 
    CodeFixProvider {
        error_codes: Vec::new(),
        fix_ids: vec![FIX_CLASS_INCORRECTLY_IMPLEMENTS_INTERFACE_FIX_ID.to_string()],
    }
}

impl LanguageService {
    pub fn get_code_actions_to_fix_class_incorrectly_implements_interface(
        &self,
        _context: &CodeFixContext,
    ) -> Vec<CodeAction> { ::tsox_core::fntrace::enter("get_code_actions_to_fix_class_incorrectly_implements_interface"); 
        Vec::new()
    }

    pub fn get_all_code_actions_to_fix_class_incorrectly_implements_interface(
        &self,
        _context: &CodeFixContext,
    ) -> super::code_actions::CombinedCodeActions { ::tsox_core::fntrace::enter("get_all_code_actions_to_fix_class_incorrectly_implements_interface"); 
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
) { ::tsox_core::fntrace::enter("add_changes"); 
}

pub fn get_changes(
    _change_tracker: &crate::ls::change::Tracker,
    _file: &Arc<tsox_frontend::ast::SourceFile>,
) -> Vec<crate::lsp::lsproto_lsp::TextEdit> { ::tsox_core::fntrace::enter("get_changes"); 
    Vec::new()
}

pub fn create_import_adder(
    _context: &CodeFixContext,
    _type_checker: &Checker,
) -> Result<(), String> { ::tsox_core::fntrace::enter("create_import_adder"); 
    Ok(())
}
