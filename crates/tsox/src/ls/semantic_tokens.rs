#![allow(dead_code)]

mod build;

use build::*;

use std::sync::Arc;

use crate::ast::SourceFile;
use crate::ast::node::LineMap;
use crate::checker::Checker;
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{DocumentUri, Range};

use super::language_service::LanguageService;
use super::types::SemanticTokens;

pub mod token_type {
    pub const NAMESPACE: u32 = 0;
    pub const CLASS: u32 = 1;
    pub const ENUM: u32 = 2;
    pub const INTERFACE: u32 = 3;
    pub const STRUCT: u32 = 4;
    pub const TYPE_PARAMETER: u32 = 5;
    pub const TYPE: u32 = 6;
    pub const PARAMETER: u32 = 7;
    pub const VARIABLE: u32 = 8;
    pub const PROPERTY: u32 = 9;
    pub const ENUM_MEMBER: u32 = 10;
    pub const DECORATOR: u32 = 11;
    pub const EVENT: u32 = 12;
    pub const FUNCTION: u32 = 13;
    pub const METHOD: u32 = 14;
    pub const MACRO: u32 = 15;
    pub const LABEL: u32 = 16;
    pub const COMMENT: u32 = 17;
    pub const STRING: u32 = 18;
    pub const KEYWORD: u32 = 19;
    pub const NUMBER: u32 = 20;
    pub const REGEXP: u32 = 21;
    pub const OPERATOR: u32 = 22;
    pub const INVALID: u32 = u32::MAX;
}

pub mod token_modifier {
    pub const DECLARATION: u32 = 1 << 0;
    pub const DEFINITION: u32 = 1 << 1;
    pub const READONLY: u32 = 1 << 2;
    pub const STATIC: u32 = 1 << 3;
    pub const DEPRECATED: u32 = 1 << 4;
    pub const ABSTRACT: u32 = 1 << 5;
    pub const ASYNC: u32 = 1 << 6;
    pub const MODIFICATION: u32 = 1 << 7;
    pub const DOCUMENTATION: u32 = 1 << 8;
    pub const DEFAULT_LIBRARY: u32 = 1 << 9;
    pub const LOCAL: u32 = 1 << 10;
}

pub struct SemanticToken {
    pub token_type: u32,
    pub token_modifier: u32,
    pub pos: usize,
    pub end: usize,
}

impl LanguageService {
    pub fn provide_semantic_tokens(&self, document_uri: &DocumentUri) -> Option<SemanticTokens> {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let checker = program.build_checker();
        let tokens = self.collect_semantic_tokens_in_range(
            &checker,
            &source_file,
            &program,
            0,
            source_file.text.len(),
        );
        let data = encode_tokens(&tokens, &source_file.line_map);
        Some(SemanticTokens { data })
    }

    pub fn provide_semantic_tokens_range(
        &self,
        document_uri: &DocumentUri,
        rng: Range,
    ) -> Option<SemanticTokens> {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let line_map = &source_file.line_map;
        let start = lsp_position_to_offset(line_map, &rng.start);
        let end = lsp_position_to_offset(line_map, &rng.end);
        let checker = program.build_checker();
        let tokens =
            self.collect_semantic_tokens_in_range(&checker, &source_file, &program, start, end);
        let data = encode_tokens(&tokens, line_map);
        Some(SemanticTokens { data })
    }

    pub fn collect_semantic_tokens_in_range(
        &self,
        checker: &Checker,
        file: &Arc<SourceFile>,
        _program: &Program,
        span_start: usize,
        span_end: usize,
    ) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();
        collect_tokens(checker, &file.node, span_start, span_end, &mut tokens);

        tokens.sort_by_key(|t| t.pos);
        tokens
    }
}

fn encode_tokens(tokens: &[SemanticToken], line_map: &LineMap) -> Vec<u32> {
    let mut data = Vec::with_capacity(tokens.len() * 5);
    let mut prev_line = 0u32;
    let mut prev_char = 0u32;

    for token in tokens {
        let start = offset_to_line_char(line_map, token.pos);
        let end = offset_to_line_char(line_map, token.end);
        let length = (token.end - token.pos) as u32;

        let delta_line = start.0 - prev_line;
        let delta_char = if delta_line == 0 {
            start.1 - prev_char
        } else {
            start.1
        };

        data.push(delta_line);
        data.push(delta_char);
        data.push(length);
        data.push(token.token_type);
        data.push(token.token_modifier);

        prev_line = start.0;
        prev_char = start.1;
        let _ = end;
    }

    data
}

fn offset_to_line_char(line_map: &LineMap, offset: usize) -> (u32, u32) {
    let line = match line_map.line_starts.binary_search(&(offset as u32)) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    };
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    (line as u32, (offset.saturating_sub(line_start)) as u32)
}

fn lsp_position_to_offset(
    line_map: &LineMap,
    position: &crate::lsp::lsproto::lsp::Position,
) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}
