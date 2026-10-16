#![allow(unused_imports)]

//! Go printer.EscapeString（tsc/internal/printer/utilities.go:182）的
//! checker 侧移植：ValueToString / LiteralType.String 渲染字面量值时
//! 转义控制字符、引号、反斜杠与 \u2028/\u2029/\u0085；0x7f 与非 ASCII
//! （NeverAsciiEscape 档）原样；\0 后随数字走 \x00 防八进制误读。
//!
//! 单独进 checker crate：tsox-emit 反向依赖 tsox-checker，不能复用其
//! printer::name_generator 同名实现。

use tsox_core::stringutil::is_digit;

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

/// Go escapedCharsMap（printer/utilities.go:42）
fn escaped_chars_map(code: u32) -> Option<&'static str> {
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

/// Go encodeUtf16EscapeSequence（printer/utilities.go:67）：大写四位十六进制
fn encode_utf16_escape_sequence(b: &mut String, code: u32) { ::tsox_core::fntrace::enter("encode_utf16_escape_sequence");
    let hex = format!("{:X}", code);
    b.push_str("\\u");
    for _ in 0..(4 - hex.len()) {
        b.push('0');
    }
    b.push_str(&hex);
}

pub fn escape_string(s: &str, quote_char: QuoteChar) -> String { ::tsox_core::fntrace::enter("escape_string");
    let mut b = String::with_capacity(s.len() + 2);
    let bytes = s.as_bytes();
    // Go escapeStringWorker（printer/utilities.go:80-178）：pos 跟踪已写
    // 末端，遇转义字符先冲刷 [pos,i) 未转义段，结尾冲刷 [pos,len) 尾段
    let mut pos = 0usize;
    let mut skip_next = false;

    for (i, ch) in s.char_indices() {
        if skip_next {
            skip_next = false;
            continue;
        }
        let code = ch as u32;
        let escape = ch == '\\'
            || (ch == '$'
                && quote_char == QuoteChar::Backtick
                && i + 1 < s.len()
                && bytes[i + 1] == b'{')
            || ch == quote_char.as_char()
            || matches!(ch, '\u{2028}' | '\u{2029}' | '\u{0085}' | '\r')
            || (ch == '\n' && quote_char != QuoteChar::Backtick)
            || code <= 0x1f;

        if !escape {
            continue;
        }

        if pos < i {
            b.push_str(&s[pos..i]);
        }

        let consumed = if ch == '\r'
            && quote_char == QuoteChar::Backtick
            && i + 1 < s.len()
            && bytes[i + 1] == b'\n'
        {
            b.push_str("\\r\\n");
            skip_next = true;
            2
        } else if ch == '\\' && is_lone_surrogate_escape_text(&s[i..]) {
            // Rust 值侧 lone surrogate 以 \uXXXX 文本表示（scanner
            // push_unicode_escape 产物），恰为 Go 对 sentinel 的转义形态，
            // 原样透传避免二次转义
            b.push_str(&s[i..i + 6]);
            6
        } else {
            if code == 0 {
                if i + 1 < s.len() && is_digit(bytes[i + 1] as char) {
                    b.push_str("\\x00");
                } else {
                    b.push_str("\\0");
                }
            } else if let Some(repl) = escaped_chars_map(code) {
                b.push_str(repl);
            } else {
                encode_utf16_escape_sequence(&mut b, code);
            }
            ch.len_utf8()
        };

        pos = i + consumed;
    }

    if pos < s.len() {
        b.push_str(&s[pos..]);
    }

    b
}

/// `\uD800`-`\uDFFF` 形态的 lone surrogate 文本（恰 6 字节）
fn is_lone_surrogate_escape_text(rest: &str) -> bool {
    let bytes = rest.as_bytes();
    let third = bytes.get(3).map(|b| b.to_ascii_uppercase());
    bytes.len() >= 6
        && bytes[0] == b'\\'
        && (bytes[1] == b'u' || bytes[1] == b'U')
        && bytes[2] == b'D'
        && matches!(third, Some(b'8' | b'9' | b'A' | b'B' | b'C' | b'D' | b'E' | b'F'))
        && bytes[4].is_ascii_hexdigit()
        && bytes[5].is_ascii_hexdigit()
}
