#![allow(dead_code, unused_imports, unused_variables)]

use super::m5q_3::get_default_commit_characters;
use super::m5q2b_3::lsproto;
use crate::ls::language_service::LanguageService;
use crate::ls::mig::m5q2_2::{CompletionItemDefaults, CompletionList};
use std::sync::Arc;

impl LanguageService {
    pub fn js_doc_completion_info(
        &self,
        position: usize,
        file: &Arc<tsox_frontend::ast::SourceFile>,
        mut items: Vec<lsproto::CompletionItem>,
    ) -> CompletionList { ::tsox_core::fntrace::enter("js_doc_completion_info"); 
        let default_commit_characters = get_default_commit_characters(false);
        let item_defaults = self.set_item_defaults(
            position,
            file,
            &mut items,
            Some(&default_commit_characters),
            None,
        );
        CompletionList {
            is_incomplete: false,
            item_defaults,
            items,
        }
    }
}
