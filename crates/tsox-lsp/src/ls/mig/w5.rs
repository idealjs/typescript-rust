#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;

use tsox_frontend::ast::{self, Node, SourceFile};

use crate::ls::lsutil_symbol_display::ScriptElementKind;
use super::m5q_3::{get_default_commit_characters, SORT_TEXT_LOCATION_PRIORITY};
use super::m5y_pc_2::PseudoNodeExt;

mod lsproto {
    pub use crate::lsp::lsproto::*;

    pub use crate::ls::mig::m5q2b_3::lsproto::CompletionItem;

    pub use crate::ls::mig::m5q2_2::{CompletionItemDefaults, CompletionList};
}

impl crate::ls::language_service::LanguageService {
    pub fn get_label_completions_at_position(
        &self,
        node: &Arc<Node>,
        file: &Arc<SourceFile>,
        position: usize,
        optional_replacement_span: Option<lsproto::Range>,
    ) -> Option<lsproto::CompletionList> { ::tsox_core::fntrace::enter("get_label_completions_at_position"); 
        let mut items = self.get_label_statement_completions(node, file, position);
        if items.is_empty() {
            return None;
        }
        let default_commit_characters = get_default_commit_characters(false);
        let item_defaults = self.set_item_defaults(
            position,
            file,
            &mut items,
            Some(&default_commit_characters),
            optional_replacement_span.as_ref(),
        );
        Some(lsproto::CompletionList {
            is_incomplete: false,
            item_defaults,
            items,
        })
    }

    pub fn get_label_statement_completions(
        &self,
        node: &Arc<Node>,
        file: &Arc<SourceFile>,
        position: usize,
    ) -> Vec<lsproto::CompletionItem> { ::tsox_core::fntrace::enter("get_label_statement_completions"); 
        let mut uniques: HashSet<String> = HashSet::new();
        let mut items = Vec::new();
        let mut current = Some(node.clone());
        while let Some(cur) = current {
            if ast::is_function_like(&cur) {
                break;
            }
            if ast::is_labeled_statement(&cur) {
                if let Some(label) = cur.label() {
                    let name = label.text().to_string();
                    if !uniques.contains(&name) {
                        uniques.insert(name.clone());
                        let lsp_item = self.create_lsp_completion_item(
                            &name,
                            "",
                            "",
                            SORT_TEXT_LOCATION_PRIORITY.to_string(),
                            ScriptElementKind::Label,
                            vec![],
                            None,
                            None,
                            None,
                            file,
                            position,
                            false,
                            false,
                            false,
                            false,
                            "",
                            None,
                            None,
                            None,
                        );
                        items.push(lsp_item);
                    }
                }
            }
            current = cur.parent();
        }
        items
    }
}
