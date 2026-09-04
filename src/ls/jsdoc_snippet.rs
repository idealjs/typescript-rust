#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile};

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
    ) -> Option<CompletionItem> {

        None
    }
}

pub fn is_potentially_valid_jsdoc_snippet_completion_position(
    _file: &Arc<SourceFile>,
    _position: usize,
) -> bool {

    false
}

pub fn get_doc_comment_template_at_position(
    _file: &Arc<SourceFile>,
    _position: usize,
    _generate_return: bool,
    _new_line: &str,
) -> Option<DocCommentTemplate> {

    None
}

pub fn template_to_snippet(template: &str, _new_line: &str) -> String {

    template.to_string()
}
