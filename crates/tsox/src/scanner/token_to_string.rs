#![allow(unused_imports)]

use super::*;

pub fn token_to_string(token: SyntaxKind) -> &'static str {
    TOKEN_TO_TEXT
        .get_or_init(|| {
            let mut m = HashMap::new();
            for (&text, &kind) in keywords().iter() {
                m.insert(kind, text);
            }
            for (&text, &kind) in punctuation().iter() {
                m.insert(kind, text);
            }
            m.insert(SyntaxKind::Identifier, "identifier");
            m.insert(SyntaxKind::EndOfFile, "end of file");
            m.insert(SyntaxKind::NumericLiteral, "numeric literal");
            m.insert(SyntaxKind::StringLiteral, "string literal");
            m.insert(SyntaxKind::BigIntLiteral, "bigint literal");
            m.insert(
                SyntaxKind::RegularExpressionLiteral,
                "regular expression literal",
            );
            m.insert(SyntaxKind::TemplateHead, "template literal");
            m.insert(SyntaxKind::TemplateMiddle, "template literal");
            m.insert(SyntaxKind::TemplateTail, "template literal");
            m.insert(
                SyntaxKind::NoSubstitutionTemplateLiteral,
                "template literal",
            );
            m.insert(SyntaxKind::Unknown, "unknown");
            m
        })
        .get(&token)
        .copied()
        .unwrap_or("")
}

#[derive(Clone)]
pub struct Scanner {
    pub(crate) text: std::sync::Arc<str>,
    pub(crate) pos: usize,
    pub(crate) end: usize,
    pub(crate) token: SyntaxKind,
    pub(crate) token_pos: usize,
    pub(crate) token_end: usize,

    pub(crate) full_start_pos: usize,
    pub(crate) preceding_line_break: bool,
    pub(crate) has_preceding_line_break: bool,

    pub(crate) binary_marker_pos: Option<usize>,

    pub(crate) token_flags: TokenFlags,

    pub(crate) skip_jsdoc_leading_asterisks: i32,
    pub(crate) error_callback: Option<ErrorCallback>,

    pub(crate) errors: Vec<ScannerError>,

    pub(crate) comment_directives: Vec<CommentDirective>,

    pub(crate) script_target: crate::core::compiler_options::ScriptTarget,

    pub(crate) language_variant: crate::ast::LanguageVariant,

    pub(crate) identifier_value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScannerError {
    pub kind: DiagnosticKind,
    pub pos: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentDirectiveKind {
    ExpectError,

    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentDirective {
    pub pos: usize,
    pub end: usize,
    pub kind: CommentDirectiveKind,
}

#[derive(Debug, Clone, Default)]
pub struct ScannerOptions {
    pub language_variant: crate::ast::LanguageVariant,
    pub skip_trivia: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ScannerState {
    pub(crate) pos: usize,
    pub(crate) end: usize,
    pub(crate) token: SyntaxKind,
    pub(crate) token_pos: usize,
    pub(crate) token_end: usize,
    pub(crate) full_start_pos: usize,
    pub(crate) preceding_line_break: bool,
    pub(crate) has_preceding_line_break: bool,
    pub(crate) binary_marker_pos: Option<usize>,
    pub(crate) token_flags: TokenFlags,
    pub(crate) skip_jsdoc_leading_asterisks: i32,
    pub(crate) errors_len: usize,
    pub(crate) comment_directives_len: usize,
}
