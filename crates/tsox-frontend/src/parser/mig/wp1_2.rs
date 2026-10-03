#![allow(unused_imports)]

//! wp1 continuation: pragma/error-span helpers

use crate::ast::*;
use crate::ast::node_source_file::FileReference;
use crate::parser::parsing_context::Parser;
use crate::scanner::{get_leading_comment_ranges, CommentRange, CommentRangeKind};
use tsox_core::core::compiler_options_kinds::ModuleKind;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics;

/// Go ast.PragmaArgument
#[derive(Debug, Clone)]
pub struct PragmaArgument {
    pub name: String,
    pub value: String,
    pub text_range: TextRange,
}

/// Go ast.Pragma
#[derive(Debug, Clone)]
pub struct Pragma {
    pub comment_range: CommentRange,
    pub name: String,
    pub args: Vec<PragmaArgument>,
}

pub(crate) fn extract_name(text: &str, mut pos: usize) -> String { ::tsox_core::fntrace::enter("extract_name"); 
    let start = pos;
    let bytes = text.as_bytes();
    while pos < bytes.len()
        && (bytes[pos].is_ascii_uppercase() || bytes[pos].is_ascii_lowercase() || bytes[pos] == b'-')
    {
        pos += 1;
    }
    text[start..pos].to_ascii_lowercase()
}

pub(crate) fn extract_quoted_string(text: &str, mut pos: usize) -> Option<String> { ::tsox_core::fntrace::enter("extract_quoted_string"); 
    let bytes = text.as_bytes();
    if pos == bytes.len() {
        return None;
    }
    let quote = bytes[pos];
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    pos += 1;
    let start = pos;
    while pos < bytes.len() && bytes[pos] != quote {
        pos += 1;
    }
    if pos == bytes.len() {
        return None;
    }
    Some(text[start..pos].to_string())
}

pub(crate) fn get_comment_pragmas(source_text: &str) -> Vec<Pragma> { ::tsox_core::fntrace::enter("get_comment_pragmas"); 
    let mut pragmas = Vec::new();
    for comment_range in get_leading_comment_ranges(source_text, 0) {
        let comment = &source_text[comment_range.pos..comment_range.end];
        pragmas.extend(extract_pragmas(comment_range, comment));
    }
    pragmas
}

pub(crate) fn extract_pragmas(comment_range: CommentRange, text: &str) -> Vec<Pragma> { ::tsox_core::fntrace::enter("extract_pragmas"); 
    if comment_range.kind != CommentRangeKind::SingleLine {
        return Vec::new();
    }
    let mut pos = 2;
    let triple_slash = text[pos..].starts_with("/");
    if triple_slash {
        pos += 1;
    }
    pos = skip_blanks(text, pos);
    if !(triple_slash && text[pos..].starts_with("<")) {
        return Vec::new();
    }
    let tag_name = extract_name(text, pos + 1);
    if tag_name != "reference" {
        return Vec::new();
    }
    pos += 10;
    let mut args = Vec::new();
    loop {
        pos = skip_blanks(text, pos);
        if text[pos..].starts_with("/>") {
            break;
        }
        let arg_name = extract_name(text, pos);
        if arg_name.is_empty() {
            break;
        }
        pos = skip_blanks(text, pos + arg_name.len());
        if !text[pos..].starts_with("=") {
            break;
        }
        pos = skip_blanks(text, pos + 1);
        let value = match extract_quoted_string(text, pos) {
            Some(value) => value,
            None => break,
        };
        args.push(PragmaArgument {
            name: arg_name,
            value: value.clone(),
            text_range: TextRange::new(comment_range.pos + pos + 1, comment_range.pos + pos + 1 + value.len()),
        });
        pos += value.len() + 2;
    }
    vec![Pragma {
        comment_range,
        name: "reference".to_string(),
        args,
    }]
}

pub(crate) fn skip_blanks(text: &str, mut pos: usize) -> usize { ::tsox_core::fntrace::enter("skip_blanks"); 
    let bytes = text.as_bytes();
    while pos < bytes.len() && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
        pos += 1;
    }
    pos
}

pub(crate) fn get_error_span_for_node(source_text: &str, node: &Node) -> TextRange { ::tsox_core::fntrace::enter("get_error_span_for_node"); 
    let mut pos = node.pos();
    if !(node.pos() == node.end() && (node.pos() as i32) >= 0 && node.kind != SyntaxKind::EndOfFile) {
        pos = crate::scanner::skip_trivia(source_text, pos);
    }
    TextRange::new(pos, node.end())
}

pub(crate) fn get_space_suggestion(expression_text: &str) -> String { ::tsox_core::fntrace::enter("get_space_suggestion"); 
    for keyword in VIABLE_KEYWORD_SUGGESTIONS {
        if expression_text.len() > keyword.len() + 2 && expression_text.starts_with(keyword) {
            return format!("{} {}", keyword, &expression_text[keyword.len()..]);
        }
    }
    String::new()
}

pub(crate) const VIABLE_KEYWORD_SUGGESTIONS: &[&str] = &[
    "abstract", "any", "as", "asserts", "assert", "boolean", "break", "case", "catch", "class",
];

impl Parser {
    pub(crate) fn process_pragmas_into_fields(
        &mut self,
        context: &mut crate::ast::node_source_file::SourceFile,
    ) { ::tsox_core::fntrace::enter("process_pragmas_into_fields");
        context.referenced_files = Vec::new();
        context.type_reference_directives = Vec::new();
        context.lib_reference_directives = Vec::new();
        for pragma in get_comment_pragmas(self.source_text()) {
            let find_arg = |name: &str| pragma.args.iter().find(|a| a.name == name).cloned();
            match pragma.name.as_str() {
                "reference" => {
                    if find_arg("no-default-lib").is_some_and(|a| a.value == "true") {
                        // Ignored.
                    } else if let Some(types) = find_arg("types") {
                        let resolution_mode = find_arg("resolution-mode").map_or(
                            ModuleKind::None,
                            |m| self.parse_resolution_mode(&m.value, m.text_range.pos(), m.text_range.end()),
                        );
                        context.type_reference_directives.push(FileReference {
                            range: types.text_range,
                            file_name: types.value,
                            resolution_mode,
                            preserve: find_arg("preserve").is_some_and(|a| a.value == "true"),
                        });
                    } else if let Some(lib) = find_arg("lib") {
                        context.lib_reference_directives.push(FileReference {
                            range: lib.text_range,
                            file_name: lib.value,
                            resolution_mode: ModuleKind::None,
                            preserve: find_arg("preserve").is_some_and(|a| a.value == "true"),
                        });
                    } else if let Some(path) = find_arg("path") {
                        context.referenced_files.push(FileReference {
                            range: path.text_range,
                            file_name: path.value,
                            resolution_mode: ModuleKind::None,
                            preserve: find_arg("preserve").is_some_and(|a| a.value == "true"),
                        });
                    } else {
                        self.parse_error_at_range(
                            TextRange::new(pragma.comment_range.pos, pragma.comment_range.end),
                            diagnostics::INVALID_REFERENCE_DIRECTIVE_SYNTAX,
                            &[],
                        );
                    }
                }
                "ts-check" | "ts-nocheck" => {
                    // CheckJsDirective 无存储（wp1b 架构裁决），暂不落字段
                }
                "jsx" | "jsxfrag" | "jsximportsource" | "jsxruntime" => {}
                other => panic!("Unhandled pragma kind: {}", other),
            }
        }
    }
}
