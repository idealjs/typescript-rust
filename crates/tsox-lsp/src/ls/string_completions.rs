#![allow(dead_code)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::Symbol;

use super::language_service::LanguageService;

pub struct CompletionsFromTypes {
    pub is_new_identifier: bool,
}

pub struct CompletionsFromProperties {
    pub symbols: Vec<Arc<Symbol>>,
    pub has_index_signature: bool,
}

pub struct PathCompletion {
    pub name: String,
    pub extension: String,
    pub text_range: TextRange,
}

pub struct StringLiteralCompletions {
    pub from_types: Option<CompletionsFromTypes>,
    pub from_properties: Option<CompletionsFromProperties>,
    pub from_paths: Vec<PathCompletion>,
}

impl LanguageService {
    pub fn get_string_literal_completions(
        &self,
        _file: &Arc<SourceFile>,
        _position: usize,
        _context_token: Option<&Arc<Node>>,
        _checker: &Checker,
    ) -> Option<StringLiteralCompletions> {
        None
    }
}
