#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::Node;

use super::language_service::LanguageService;
use super::types::VsOnAutoInsertParams;

#[derive(Debug, Clone, Default)]
pub struct VsOnAutoInsertResponseItem {
    pub text_edit_format: u32,
    pub text_edit: crate::lsp::lsproto::lsp::TextEdit,
}

impl LanguageService {
    pub fn provide_on_auto_insert(
        &self,
        _params: &VsOnAutoInsertParams,
    ) -> Option<VsOnAutoInsertResponseItem> {
        None
    }
}

pub fn is_unclosed_tag(_element: &Arc<Node>) -> bool {
    false
}

pub fn is_unclosed_fragment(_fragment: &Arc<Node>) -> bool {
    false
}
