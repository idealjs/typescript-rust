use tsox_core::stringutil::mig::m3m_2::decode_js_string_rune;


pub const GET_LITERAL_TEXT_FLAGS_NONE: u32 = 0;
pub const GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE: u32 = 1 << 0;
pub const GET_LITERAL_TEXT_FLAGS_JSX_ATTRIBUTE_ESCAPE: u32 = 1 << 1;
pub const GET_LITERAL_TEXT_FLAGS_TERMINATE_UNTERMINATED_LITERALS: u32 = 1 << 2;
pub const GET_LITERAL_TEXT_FLAGS_ALLOW_NUMERIC_SEPARATOR: u32 = 1 << 3;

pub type GetLiteralTextFlags = u32;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuoteChar {
    SingleQuote,
    DoubleQuote,
    Backtick,
}

impl QuoteChar {
    pub fn as_char(self) -> char { ::tsox_core::fntrace::enter("as_char"); 
        match self {
            QuoteChar::SingleQuote => '\'',
            QuoteChar::DoubleQuote => '"',
            QuoteChar::Backtick => '`',
        }
    }
}

pub fn jsx_escaped_chars_map(ch: char) -> Option<&'static str> { ::tsox_core::fntrace::enter("jsx_escaped_chars_map"); 
    match ch {
        '"' => Some("&quot;"),
        '\'' => Some("&apos;"),
        _ => None,
    }
}

pub fn escaped_chars_map(ch: char) -> Option<&'static str> { ::tsox_core::fntrace::enter("escaped_chars_map"); 
    match ch {
        '\t' => Some("\\t"),
        '\u{0b}' => Some("\\v"),
        '\u{0c}' => Some("\\f"),
        '\u{08}' => Some("\\b"),
        '\r' => Some("\\r"),
        '\n' => Some("\\n"),
        '\\' => Some("\\\\"),
        '"' => Some("\\\""),
        '\'' => Some("\\\'"),
        '`' => Some("\\`"),
        '$' => Some("\\$"),
        '\u{2028}' => Some("\\u2028"),
        '\u{2029}' => Some("\\u2029"),
        '\u{0085}' => Some("\\u0085"),
        _ => None,
    }
}

pub fn encode_jsx_character_entity(b: &mut String, char_code: u32) { ::tsox_core::fntrace::enter("encode_jsx_character_entity"); 
    let hex_char_code = format!("{:X}", char_code);
    b.push_str("&#x");
    b.push_str(&hex_char_code);
    b.push(';');
}

pub fn encode_utf16_escape_sequence(b: &mut String, char_code: u32) { ::tsox_core::fntrace::enter("encode_utf16_escape_sequence"); 
    let hex_char_code = format!("{:X}", char_code);
    b.push_str("\\u");
    for _ in hex_char_code.len()..4 {
        b.push('0');
    }
    b.push_str(&hex_char_code);
}

pub fn escape_string_worker(s: &str, quote_char: QuoteChar, flags: GetLiteralTextFlags, b: &mut String) { ::tsox_core::fntrace::enter("escape_string_worker"); 
    let bytes = s.as_bytes();
    let mut pos = 0;
    let mut i = 0;
    while i < bytes.len() {
        let (ch, mut size) = decode_js_string_rune(&s[i..]);

        let mut escape = false;
        if (0xD800..=0xDFFF).contains(&ch) {
            escape = true;
        } else if ch == 0xFFFD && size == 1 {
            escape = true;
        }

        if ch == '\\' as u32 {
            if flags & GET_LITERAL_TEXT_FLAGS_JSX_ATTRIBUTE_ESCAPE == 0 {
                escape = true;
            }
        } else if ch == '$' as u32 {
            if quote_char == QuoteChar::Backtick && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                escape = true;
            }
        } else if ch == quote_char.as_char() as u32
            || ch == 0x2028
            || ch == 0x2029
            || ch == 0x0085
            || ch == '\r' as u32
        {
            escape = true;
        } else if ch == '\n' as u32 {
            if quote_char != QuoteChar::Backtick {
                escape = true;
            }
        } else if ch <= 0x001f || (flags & GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE == 0 && ch > 0x007f) {
            escape = true;
        }

        if escape {
            if pos < i {
                b.push_str(&s[pos..i]);
            }

            if flags & GET_LITERAL_TEXT_FLAGS_JSX_ATTRIBUTE_ESCAPE != 0 {
                if ch == 0 {
                    b.push_str("&#0;");
                } else if let Some(match_str) =
                    jsx_escaped_chars_map(char::from_u32(ch).unwrap_or('\u{FFFD}'))
                {
                    b.push_str(match_str);
                } else {
                    encode_jsx_character_entity(b, ch);
                }
            } else if ch == '\r' as u32
                && quote_char == QuoteChar::Backtick
                && i + 1 < bytes.len()
                && bytes[i + 1] == b'\n'
            {
                size += 1;
                b.push_str("\\r\\n");
            } else if ch > 0xffff {
                let ch = ch - 0x10000;
                encode_utf16_escape_sequence(b, ((ch & 0b11111111110000000000) >> 10) + 0xD800);
                encode_utf16_escape_sequence(b, (ch & 0b00000000001111111111) + 0xDC00);
            } else if (0xD800..=0xDFFF).contains(&ch) {
                encode_utf16_escape_sequence(b, ch);
            } else if ch == 0 {
                if i + 1 < bytes.len() && (bytes[i + 1] as char).is_ascii_digit() {
                    b.push_str("\\x00");
                } else {
                    b.push_str("\\0");
                }
            } else if let Some(match_str) = escaped_chars_map(char::from_u32(ch).unwrap_or('\u{FFFD}')) {
                b.push_str(match_str);
            } else {
                encode_utf16_escape_sequence(b, ch);
            }
            pos = i + size;
        }

        i += size;
    }

    if pos < i {
        b.push_str(&s[pos..]);
    }
}

pub fn escape_string(s: &str, quote_char: QuoteChar) -> String { ::tsox_core::fntrace::enter("escape_string"); 
    let mut b = String::with_capacity(s.len() + 2);
    escape_string_worker(s, quote_char, GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE, &mut b);
    b
}

pub fn escape_non_ascii_string(s: &str, quote_char: QuoteChar) -> String { ::tsox_core::fntrace::enter("escape_non_ascii_string"); 
    let mut b = String::with_capacity(s.len() + 2);
    escape_string_worker(s, quote_char, GET_LITERAL_TEXT_FLAGS_NONE, &mut b);
    b
}

pub fn escape_jsx_attribute_string(s: &str, quote_char: QuoteChar) -> String { ::tsox_core::fntrace::enter("escape_jsx_attribute_string"); 
    let mut b = String::with_capacity(s.len() + 2);
    escape_string_worker(
        s,
        quote_char,
        GET_LITERAL_TEXT_FLAGS_JSX_ATTRIBUTE_ESCAPE | GET_LITERAL_TEXT_FLAGS_NEVER_ASCII_ESCAPE,
        &mut b,
    );
    b
}

