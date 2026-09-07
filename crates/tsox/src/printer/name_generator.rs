#![allow(unused_imports)]

use super::*;

impl Default for NameGenerator {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn has_leading_hash(text: &str) -> bool {
    text.starts_with('#')
}

pub(crate) fn remove_leading_hash(text: &str) -> &str {
    if has_leading_hash(text) {
        &text[1..]
    } else {
        text
    }
}

pub(crate) fn ensure_leading_hash(text: &str) -> String {
    if has_leading_hash(text) {
        text.to_string()
    } else {
        format!("#{text}")
    }
}

pub(crate) fn format_generated_name(
    private_name: bool,
    prefix: &str,
    base: &str,
    suffix: &str,
) -> String {
    let name = format!(
        "{}{}{}",
        remove_leading_hash(prefix),
        remove_leading_hash(base),
        remove_leading_hash(suffix)
    );
    if private_name {
        ensure_leading_hash(&name)
    } else {
        name
    }
}

pub(crate) fn make_identifier_from_module_name(module_name: &str) -> String {
    let base = crate::tspath::get_base_file_name(module_name);
    let mut result = String::new();
    let bytes = base.as_bytes();
    let mut start = 0;
    let mut pos = 0;
    while pos < bytes.len() {
        let ch = bytes[pos] as char;
        if pos == 0 && ch.is_ascii_digit() {
            result.push('_');
        } else if !is_ascii_word_character(ch) {
            if start < pos {
                result.push_str(&base[start..pos]);
            }
            result.push('_');
            start = pos + 1;
        }
        pos += 1;
    }
    if start < pos {
        result.push_str(&base[start..pos]);
    }
    if result.chars().last().map(|c| c == '_').unwrap_or(false) {
        result.pop();
    }
    result
}

pub(crate) fn is_ascii_word_character(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch.is_ascii_digit() || ch == '_'
}

pub(crate) fn get_external_module_name(node: &Arc<Node>) -> Option<String> {
    match &node.data {
        crate::ast::node_data_generated::NodeData::ImportDeclaration(d) => {
            Some(d.module_specifier.text().to_string())
        }
        crate::ast::node_data_generated::NodeData::ExportDeclaration(d) => {
            d.module_specifier.as_ref().map(|n| n.text().to_string())
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteChar {
    SingleQuote,
    DoubleQuote,
    Backtick,
}

impl QuoteChar {
    pub(crate) fn as_char(self) -> char {
        match self {
            QuoteChar::SingleQuote => '\'',
            QuoteChar::DoubleQuote => '"',
            QuoteChar::Backtick => '`',
        }
    }
}

bitflags::bitflags! {

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct GetLiteralTextFlags: u32 {
        const NONE = 0;
        const NEVER_ASCII_ESCAPE = 1 << 0;
        const JSX_ATTRIBUTE_ESCAPE = 1 << 1;
        const TERMINATE_UNTERMINATED_LITERALS = 1 << 2;
        const ALLOW_NUMERIC_SEPARATOR = 1 << 3;
    }
}

pub(crate) fn encode_jsx_character_entity(b: &mut String, ch: char) {
    b.push_str("&#x");
    b.push_str(&format!("{:X}", ch as u32));
    b.push(';');
}

pub(crate) fn encode_utf16_escape_sequence_u32(b: &mut String, code: u32) {
    let hex = format!("{:X}", code);
    b.push_str("\\u");
    for _ in 0..(4 - hex.len()) {
        b.push('0');
    }
    b.push_str(&hex);
}

pub(crate) fn encode_utf16_escape_sequence(b: &mut String, ch: char) {
    encode_utf16_escape_sequence_u32(b, ch as u32);
}

pub(crate) fn jsx_escaped_chars_map(code: u32) -> Option<&'static str> {
    match code {
        0x22 => Some("&quot;"),
        0x27 => Some("&apos;"),
        _ => None,
    }
}

pub(crate) fn escaped_chars_map(code: u32) -> Option<&'static str> {
    match code {
        0x09 => Some("\\t"),
        0x0b => Some("\\v"),
        0x0c => Some("\\f"),
        0x08 => Some("\\b"),
        0x0d => Some("\\r"),
        0x0a => Some("\\n"),
        0x5c => Some("\\\\"),
        0x22 => Some("\\\""),
        0x27 => Some("\\'"),
        0x60 => Some("\\`"),
        0x24 => Some("\\$"),
        0x2028 => Some("\\u2028"),
        0x2029 => Some("\\u2029"),
        0x0085 => Some("\\u0085"),
        _ => None,
    }
}

pub(crate) fn escape_string_worker(
    s: &str,
    quote_char: QuoteChar,
    flags: GetLiteralTextFlags,
    b: &mut String,
) {
    let bytes = s.as_bytes();
    let mut pos = 0usize;

    for (i, ch) in s.char_indices() {
        let code = ch as u32;
        let size = ch.len_utf8();
        let mut actual_size = size;
        let mut escape = false;

        if (0xD800..=0xDFFF).contains(&code) {
            escape = true;
        }

        if !escape {
            if ch == '\\' {
                if !flags.contains(GetLiteralTextFlags::JSX_ATTRIBUTE_ESCAPE) {
                    escape = true;
                }
            } else if ch == '$'
                && quote_char == QuoteChar::Backtick
                && i + 1 < s.len()
                && bytes[i + 1] == b'{'
            {
                escape = true;
            } else if ch == quote_char.as_char()
                || matches!(ch, '\u{2028}' | '\u{2029}' | '\u{0085}' | '\r')
            {
                escape = true;
            } else if ch == '\n' {
                if quote_char != QuoteChar::Backtick {
                    escape = true;
                }
            } else if code <= 0x1f
                || (!flags.contains(GetLiteralTextFlags::NEVER_ASCII_ESCAPE) && code > 0x7f)
            {
                escape = true;
            }
        }

        if escape {
            if pos < i {
                b.push_str(&s[pos..i]);
            }

            if flags.contains(GetLiteralTextFlags::JSX_ATTRIBUTE_ESCAPE) {
                if code == 0 {
                    b.push_str("&#0;");
                } else if let Some(repl) = jsx_escaped_chars_map(code) {
                    b.push_str(repl);
                } else {
                    encode_jsx_character_entity(b, ch);
                }
            } else if ch == '\r'
                && quote_char == QuoteChar::Backtick
                && i + 1 < s.len()
                && bytes[i + 1] == b'\n'
            {
                actual_size += 1;
                b.push_str("\\r\\n");
            } else if code > 0xffff {
                let adjusted = code - 0x10000;
                encode_utf16_escape_sequence_u32(b, ((adjusted >> 10) & 0x3ff) + 0xd800);
                encode_utf16_escape_sequence_u32(b, (adjusted & 0x3ff) + 0xdc00);
            } else if (0xD800..=0xDFFF).contains(&code) {
                encode_utf16_escape_sequence(b, ch);
            } else if code == 0 {
                if i + 1 < s.len() && stringutil::is_digit(bytes[i + 1] as char) {
                    b.push_str("\\x00");
                } else {
                    b.push_str("\\0");
                }
            } else if let Some(repl) = escaped_chars_map(code) {
                b.push_str(repl);
            } else {
                encode_utf16_escape_sequence(b, ch);
            }

            pos = i + actual_size;
        }
    }

    if pos < s.len() {
        b.push_str(&s[pos..]);
    }
}

pub fn escape_string(s: &str, quote_char: QuoteChar) -> String {
    let mut b = String::with_capacity(s.len() + 2);
    escape_string_worker(
        s,
        quote_char,
        GetLiteralTextFlags::NEVER_ASCII_ESCAPE,
        &mut b,
    );
    b
}

pub fn escape_non_ascii_string(s: &str, quote_char: QuoteChar) -> String {
    let mut b = String::with_capacity(s.len() + 2);
    escape_string_worker(s, quote_char, GetLiteralTextFlags::NONE, &mut b);
    b
}

pub fn escape_jsx_attribute_string(s: &str, quote_char: QuoteChar) -> String {
    let mut b = String::with_capacity(s.len() + 2);
    escape_string_worker(
        s,
        quote_char,
        GetLiteralTextFlags::JSX_ATTRIBUTE_ESCAPE | GetLiteralTextFlags::NEVER_ASCII_ESCAPE,
        &mut b,
    );
    b
}

pub(crate) fn decode_char_at(text: &str, pos: usize) -> (char, usize) {
    let c = text[pos..].chars().next().unwrap();
    (c, c.len_utf8())
}
