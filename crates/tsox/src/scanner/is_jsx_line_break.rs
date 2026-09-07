#![allow(unused_imports)]

use super::*;

pub(crate) fn is_jsx_line_break(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

pub(crate) fn is_jsx_whitespace_like(c: char) -> bool {
    matches!(c, '\t' | '\x0B' | '\x0C' | ' ' | '\u{A0}' | '\u{FEFF}') || c.is_whitespace()
}

pub(crate) fn is_identifier_or_keyword_token(token: SyntaxKind) -> bool {
    token == SyntaxKind::Identifier || is_keyword(token)
}

pub(crate) fn is_keyword(token: SyntaxKind) -> bool {
    crate::ast::node_data_generated::is_keyword_kind(token)
}

pub(crate) fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t'
            | '\n'
            | '\r'
            | '\x0B'
            | '\x0C'
            | '\u{85}'
            | '\u{A0}'
            | '\u{1680}'
            | '\u{2000}'
            | '\u{2001}'
            | '\u{2002}'
            | '\u{2003}'
            | '\u{2004}'
            | '\u{2005}'
            | '\u{2006}'
            | '\u{2007}'
            | '\u{2008}'
            | '\u{2009}'
            | '\u{200A}'
            | '\u{200B}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
            | '\u{FEFF}'
    )
}

pub(crate) fn is_line_break(c: char) -> bool {
    c == '\n' || c == '\r'
}

pub(crate) const REG_EXP_FLAG_G: u16 = 1 << 0;
pub(crate) const REG_EXP_FLAG_I: u16 = 1 << 1;
pub(crate) const REG_EXP_FLAG_M: u16 = 1 << 2;
pub(crate) const REG_EXP_FLAG_S: u16 = 1 << 3;
pub(crate) const REG_EXP_FLAG_U: u16 = 1 << 4;
pub(crate) const REG_EXP_FLAG_Y: u16 = 1 << 5;
pub(crate) const REG_EXP_FLAG_D: u16 = 1 << 6;
pub(crate) const REG_EXP_FLAG_V: u16 = 1 << 7;

pub(crate) fn reg_exp_flag_bit(c: char) -> Option<u16> {
    match c {
        'g' => Some(REG_EXP_FLAG_G),
        'i' => Some(REG_EXP_FLAG_I),
        'm' => Some(REG_EXP_FLAG_M),
        's' => Some(REG_EXP_FLAG_S),
        'u' => Some(REG_EXP_FLAG_U),
        'y' => Some(REG_EXP_FLAG_Y),
        'd' => Some(REG_EXP_FLAG_D),
        'v' => Some(REG_EXP_FLAG_V),
        _ => None,
    }
}

pub(crate) fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

pub(crate) fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

pub(crate) fn is_octal_digit(c: char) -> bool {
    ('0'..='7').contains(&c)
}

pub(crate) fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic()
        || c == '_'
        || c == '$'
        || (!c.is_ascii() && is_unicode_identifier_start(c))
}

pub fn is_identifier_part(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || c == '_'
        || c == '$'
        || (!c.is_ascii() && is_unicode_identifier_part(c))
}

pub(crate) fn is_unicode_identifier_start(c: char) -> bool {
    unicode_ident::is_xid_start(c)
}

pub(crate) fn is_unicode_identifier_part(c: char) -> bool {
    unicode_ident::is_xid_continue(c) || c == '\u{200C}' || c == '\u{200D}'
}

pub(crate) fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('v') => result.push('\u{000B}'),
                Some('0') => result.push('\0'),
                Some('x') => {
                    let hex: String = chars.by_ref().take(2).collect();
                    if let Ok(n) = u32::from_str_radix(&hex, 16) {
                        result.push(char::from_u32(n).unwrap_or('\u{FFFD}'));
                    }
                }
                Some('u') => {
                    if chars.peek() == Some(&'{') {
                        chars.next();
                        let hex: String = chars.by_ref().take_while(|c| *c != '}').collect();
                        if let Ok(n) = u32::from_str_radix(&hex, 16) {
                            result.push(char::from_u32(n).unwrap_or('\u{FFFD}'));
                        }
                    } else {
                        let hex: String = chars.by_ref().take(4).collect();
                        if let Ok(n) = u32::from_str_radix(&hex, 16) {
                            result.push(char::from_u32(n).unwrap_or('\u{FFFD}'));
                        }
                    }
                }
                Some('\\') => result.push('\\'),
                Some('\'') => result.push('\''),
                Some('"') => result.push('"'),
                Some('`') => result.push('`'),
                Some('\n') => {}
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentRangeKind {
    SingleLine,
    MultiLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentRange {
    pub pos: usize,
    pub end: usize,
    pub kind: CommentRangeKind,
    pub has_trailing_new_line: bool,
}

pub(crate) fn decode_char(text: &str, pos: usize) -> (char, usize) {
    let c = text[pos..].chars().next().unwrap();
    (c, c.len_utf8())
}

pub(crate) fn is_whitespace_like(c: char) -> bool {
    matches!(c, '\t' | '\x0B' | '\x0C' | ' ' | '\u{A0}' | '\u{FEFF}') || c.is_whitespace()
}

pub(crate) fn is_whitespace_single_line(c: char) -> bool {
    matches!(c, '\t' | '\x0B' | '\x0C' | ' ' | '\u{A0}' | '\u{FEFF}')
}

pub(crate) fn has_jsdoc_tag(text: &str, names: &[&str]) -> bool {
    for &name in names {
        if !text.starts_with(name) {
            continue;
        }
        if text.len() == name.len() {
            return true;
        }
        let ch = text.as_bytes()[name.len()] as char;
        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' || ch == '}' || ch == '*' {
            return true;
        }
    }
    false
}

pub(crate) fn scan_jsdoc_comment_for_tags(comment_text: &str) -> TokenFlags {
    let mut flags = TOKEN_FLAGS_NONE;
    let mut rest = comment_text;
    loop {
        let i = match rest.find('@') {
            Some(i) => i,
            None => return flags,
        };
        rest = &rest[i + 1..];
        if !token_flags_contains(flags, TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED)
            && has_jsdoc_tag(rest, &["deprecated"])
        {
            flags |= TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED;
        }
        if !token_flags_contains(flags, TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK)
            && has_jsdoc_tag(rest, &["see", "link", "linkcode", "linkplain"])
        {
            flags |= TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK;
        }
        if token_flags_contains(
            flags,
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED
                | TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK,
        ) {
            return flags;
        }
    }
}

pub(crate) fn is_shebang_trivia(text: &str, pos: usize) -> bool {
    if text.len() < 2 {
        return false;
    }
    debug_assert_eq!(
        pos, 0,
        "shebangs check must only be done at the start of the file"
    );
    text.as_bytes()[0] == b'#' && text.as_bytes()[1] == b'!'
}

pub(crate) fn scan_shebang_trivia(text: &str, pos: usize) -> usize {
    let text_len = text.len();
    let mut pos = pos + 2;
    while pos < text_len {
        let (ch, size) = decode_char(text, pos);
        if is_line_break(ch) {
            break;
        }
        pos += size;
    }
    pos
}

pub fn get_shebang(text: &str) -> &str {
    if !is_shebang_trivia(text, 0) {
        return "";
    }
    let end = scan_shebang_trivia(text, 0);
    &text[..end]
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SkipTriviaOptions {
    pub stop_after_line_break: bool,

    pub stop_at_comments: bool,

    pub in_jsdoc: bool,
}

pub(crate) const MERGE_CONFLICT_MARKER_LENGTH: usize = 7;
