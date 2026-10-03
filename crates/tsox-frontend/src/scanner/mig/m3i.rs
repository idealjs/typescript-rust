#![allow(dead_code)]

use crate::ast::node_flags::NodeFlags;
use crate::ast::node_node::Node;
use crate::ast::node_node_list::NodeList;
use crate::ast::node_source_file::SourceFile;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::{is_identifier, is_jsdoc_type_expression, is_string_literal, is_type_node};
use crate::scanner::error_callback::{string_to_keyword, TOKEN_FLAGS_SINGLE_QUOTE, TOKEN_FLAGS_UNTERMINATED};
use crate::scanner::token_to_string::Scanner;
use crate::scanner::DiagnosticKind;
use crate::scanner::{is_identifier_start, skip_trivia};
use super::m4d_2::is_identifier_part_ex;
use crate::ast::utilities_navigation::get_source_file_of_node;
use crate::ast::utilities_synthesized::node_is_missing;
use crate::ast::LanguageVariant;
use crate::ast::node_data_generated::NodeData;
use std::sync::Arc;
use tsox_core::core::mig::m3j::compute_ecma_line_starts;
use tsox_core::stringutil;

impl Scanner {
    pub(crate) fn scan_template_and_set_token_value(
        &mut self,
        should_emit_invalid_escape_error: bool,
    ) -> SyntaxKind { ::tsox_core::fntrace::enter("scan_template_and_set_token_value"); 
        let started_with_backtick = self.char() == Some('`');
        self.pos += 1;
        let mut start = self.pos;
        let mut parts: Vec<String> = Vec::with_capacity(4);
        let token;
        loop {
            while self.pos < self.end {
                let b = self.text.as_bytes()[self.pos];
                if b != b'`' && b != b'$' && b != b'\\' && b != b'\r' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let ch = self.char();
            if self.pos >= self.end || ch == Some('`') {
                parts.push(self.text[start..self.pos].to_string());
                if self.pos < self.end {
                    self.pos += 1;
                } else {
                    self.token_flags |= TOKEN_FLAGS_UNTERMINATED;
                    self.report_error(DiagnosticKind::UnterminatedTemplateLiteral, self.pos, 0);
                }
                token = if started_with_backtick {
                    SyntaxKind::NoSubstitutionTemplateLiteral
                } else {
                    SyntaxKind::TemplateTail
                };
                break;
            }
            if ch == Some('$') && self.char_at(1) == Some('{') {
                parts.push(self.text[start..self.pos].to_string());
                self.pos += 2;
                token = if started_with_backtick {
                    SyntaxKind::TemplateHead
                } else {
                    SyntaxKind::TemplateMiddle
                };
                break;
            }
            if ch == Some('\\') {
                parts.push(self.text[start..self.pos].to_string());
                let escape_start = self.pos;
                self.scan_escape_sequence(should_emit_invalid_escape_error);
                parts.push(self.text[escape_start..self.pos].to_string());
                start = self.pos;
                continue;
            }
            if ch == Some('\r') {
                parts.push(self.text[start..self.pos].to_string());
                self.pos += 1;
                if self.char() == Some('\n') {
                    self.pos += 1;
                }
                parts.push("\n".to_string());
                start = self.pos;
                continue;
            }
            self.pos += 1;
        }
        self.identifier_value = Some(parts.join(""));
        token
    }
}

pub fn identifier_to_keyword_kind(node: &Arc<Node>) -> SyntaxKind { ::tsox_core::fntrace::enter("identifier_to_keyword_kind"); 
    string_to_keyword(&node.text()).unwrap_or(SyntaxKind::Identifier)
}

pub fn get_source_text_of_node_from_source_file(
    source_file: &SourceFile,
    node: &Arc<Node>,
    include_trivia: bool,
) -> String { ::tsox_core::fntrace::enter("get_source_text_of_node_from_source_file"); 
    get_text_of_node_from_source_text(&source_file.text, node, include_trivia)
}

pub(crate) fn is_jsdoc_type_expression_or_child(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_jsdoc_type_expression_or_child"); 
    if is_jsdoc_type_expression(node) {
        return true;
    }
    if !node.flags.intersects(NodeFlags::JSDoc | NodeFlags::Reparsed) {
        return false;
    }
    let mut current = Some(node.clone());
    while let Some(c) = current {
        if is_type_node(&c) {
            return true;
        }
        current = c.parent();
    }
    false
}

pub fn normalize_jsdoc_type_source_text(text: &str) -> String { ::tsox_core::fntrace::enter("normalize_jsdoc_type_source_text"); 
    let line_starts = compute_ecma_line_starts(text);
    if line_starts.len() == 1 {
        return strip_leading_jsdoc_comment(text);
    }

    let mut result = String::with_capacity(text.len());
    let new_line = "\n";
    for (i, line_start) in line_starts.iter().enumerate() {
        if i > 0 {
            result.push_str(new_line);
        }
        let line_end = if i + 1 < line_starts.len() {
            line_starts[i + 1] as usize
        } else {
            text.len()
        };
        let line = text[*line_start as usize..line_end].trim_end_matches(|c| stringutil::is_line_break(c));
        result.push_str(&strip_leading_jsdoc_comment(line));
    }
    result
}

pub(crate) fn strip_leading_jsdoc_comment(line: &str) -> String { ::tsox_core::fntrace::enter("strip_leading_jsdoc_comment"); 
    let line = line.trim_start_matches(|c| stringutil::is_white_space_like(c));
    let line = if !line.is_empty() && line.as_bytes()[0] == b'*' {
        &line[1..]
    } else {
        line
    };
    line.trim_start_matches(|c| stringutil::is_white_space_like(c)).to_string()
}

pub fn get_text_of_node_from_source_text(
    source_text: &str,
    node: &Arc<Node>,
    include_trivia: bool,
) -> String { ::tsox_core::fntrace::enter("get_text_of_node_from_source_text"); 
    if node_is_missing(Some(node)) {
        return String::new();
    }
    let mut pos = node.pos();
    if !include_trivia {
        pos = skip_trivia(source_text, pos);
    }
    let mut text = source_text[pos..node.end()].to_string();
    if is_jsdoc_type_expression_or_child(node) {
        text = normalize_jsdoc_type_source_text(&text);
    }
    if node.flags.contains(NodeFlags::ReparserTransformedLiteral) {
        if is_string_literal(node) {
            let single_quoted = matches!(
                &node.data,
                NodeData::StringLiteral(d) if d.token_flags & TOKEN_FLAGS_SINGLE_QUOTE != 0
            );
            if single_quoted {
                return format!("'{}'", text);
            }
            return format!("\"{}\"", text);
        } else if is_identifier(node) {
            return node.text().to_string();
        }
        debug_fail_bad_syntax_kind(node, "Unexpected reparser-transformed node kind");
    }
    text
}

fn debug_fail_bad_syntax_kind(node: &Node, message: &str) -> ! { ::tsox_core::fntrace::enter("debug_fail_bad_syntax_kind"); 
    tsox_core::debug::fail(&format!(
        "{}\nNode {:?} was unexpected.",
        message, node.kind
    ))
}

pub fn get_text_of_node(node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("get_text_of_node"); 
    let source_file = get_source_file_of_node(node).expect("node has no source file");
    let NodeData::SourceFile(_) = &source_file.data else {
        tsox_core::debug::fail("node has no source file")
    };
    get_text_of_node_from_source_text(node.text(), node, false)
}

pub fn get_text_of_jsdoc_comment(comment: Option<&NodeList>) -> String { ::tsox_core::fntrace::enter("get_text_of_jsdoc_comment"); 
    let Some(comment) = comment else {
        return String::new();
    };
    let mut b = String::new();
    for n in &comment.nodes {
        match n.kind {
            SyntaxKind::JSDocText => b.push_str(n.text()),
            SyntaxKind::JSDocLink | SyntaxKind::JSDocLinkCode | SyntaxKind::JSDocLinkPlain => {
                b.push_str(&get_text_of_node(n));
            }
            _ => {}
        }
    }
    b.trim_end().to_string()
}

pub fn declaration_name_to_string(name: Option<&Arc<Node>>) -> String { ::tsox_core::fntrace::enter("declaration_name_to_string"); 
    match name {
        None => "(Missing)".to_string(),
        Some(name) if name.pos() == name.end() => "(Missing)".to_string(),
        Some(name) => get_text_of_node(name),
    }
}

pub fn is_identifier_text(name: &str, language_variant: LanguageVariant) -> bool { ::tsox_core::fntrace::enter("is_identifier_text"); 
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !is_identifier_start(first) {
        return false;
    }
    for ch in chars {
        if !is_identifier_part_ex(ch, language_variant) {
            return false;
        }
    }
    true
}

pub fn is_intrinsic_jsx_name(name: &str) -> bool { ::tsox_core::fntrace::enter("is_intrinsic_jsx_name"); 
    let b = name.as_bytes().first().copied().unwrap_or(0);
    !name.is_empty() && (b.is_ascii_lowercase() || name.contains('-'))
}
