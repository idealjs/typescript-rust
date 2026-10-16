#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::Node;

use super::language_service::LanguageService;
use super::types::VsOnAutoInsertParams;

#[derive(Debug, Clone, Default)]
pub struct VsOnAutoInsertResponseItem {
    pub text_edit_format: u32,
    pub text_edit: crate::lsp::lsproto_lsp::TextEdit,
}

impl LanguageService {
    pub fn provide_on_auto_insert(
        &self,
        _params: &VsOnAutoInsertParams,
    ) -> Option<VsOnAutoInsertResponseItem> { ::tsox_core::fntrace::enter("provide_on_auto_insert"); 
        None
    }
}

pub fn is_unclosed_tag(_element: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_unclosed_tag"); 
    false
}

pub fn is_unclosed_fragment(_fragment: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_unclosed_fragment"); 
    false
}
