#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::Node;
use tsox_frontend::ast::SourceFile;

use super::language_service::LanguageService;
use super::types::CompletionItem;

pub struct DocCommentTemplate {
    pub new_text: String,
}

pub struct CommentOwnerInfo {
    pub comment_owner: Option<Arc<Node>>,
    pub has_return: bool,
}

impl LanguageService {
    pub fn get_jsdoc_snippet_completion(
        &self,
        _file: &Arc<SourceFile>,
        _position: usize,
    ) -> Option<CompletionItem> { ::tsox_core::fntrace::enter("get_jsdoc_snippet_completion"); 
        None
    }
}

pub fn is_potentially_valid_jsdoc_snippet_completion_position(
    _file: &Arc<SourceFile>,
    _position: usize,
) -> bool { ::tsox_core::fntrace::enter("is_potentially_valid_jsdoc_snippet_completion_position"); 
    false
}

pub fn get_doc_comment_template_at_position(
    _file: &Arc<SourceFile>,
    _position: usize,
    _generate_return: bool,
    _new_line: &str,
) -> Option<DocCommentTemplate> { ::tsox_core::fntrace::enter("get_doc_comment_template_at_position"); 
    None
}

pub fn template_to_snippet(template: &str, _new_line: &str) -> String { ::tsox_core::fntrace::enter("template_to_snippet"); 
    template.to_string()
}
