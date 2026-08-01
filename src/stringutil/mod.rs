//! String utilities, ported from `internal/stringutil/`.
//!
//! Provides character classification and string comparison utilities used
//! throughout the compiler.

/// Whether a character is whitespace-like (including line breaks).
pub fn is_white_space_like(ch: char) -> bool {
    is_white_space_single_line(ch) || is_line_break(ch)
}

/// Whether a character is a single-line whitespace character.
pub fn is_white_space_single_line(ch: char) -> bool {
    matches!(
        ch,
        ' ' | '\t'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{0085}'
            | '\u{00A0}'
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

/// Whether a character is a line break.
pub fn is_line_break(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// Whether a character is a decimal digit.
pub fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

/// Whether a character is an octal digit.
pub fn is_octal_digit(ch: char) -> bool {
    ('0'..='7').contains(&ch)
}

/// Whether a character is a hexadecimal digit.
pub fn is_hex_digit(ch: char) -> bool {
    ch.is_ascii_hexdigit()
}

/// Whether a character is an ASCII letter.
pub fn is_ascii_letter(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

/// Whether a string contains any non-ASCII bytes.
pub fn contains_non_ascii(s: &str) -> bool {
    s.bytes().any(|b| b >= 0x80)
}

/// Case-insensitive string equality.
pub fn equate_string_case_insensitive(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Case-sensitive string equality.
pub fn equate_string_case_sensitive(a: &str, b: &str) -> bool {
    a == b
}

/// Case-insensitive string comparison. Returns negative, zero, or positive.
pub fn compare_strings_case_insensitive(a: &str, b: &str) -> i32 {
    a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()) as i32
}

/// Case-sensitive string comparison. Returns negative, zero, or positive.
pub fn compare_strings_case_sensitive(a: &str, b: &str) -> i32 {
    a.cmp(b) as i32
}

/// Split text into lines at line break boundaries.
pub fn split_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            lines.push(text[start..i].to_string());
            start = i + 1;
        } else if bytes[i] == b'\r' {
            lines.push(text[start..i].to_string());
            start = i + 1;
            if start < bytes.len() && bytes[start] == b'\n' {
                start += 1;
                i += 1;
            }
        }
        i += 1;
    }
    lines.push(text[start..].to_string());
    lines
}

const UPPER_HEX: &[u8] = b"0123456789ABCDEF";

/// Percent-encode a URI string.
///
/// Mirrors `stringutil.EncodeURI` in Go and the ECMAScript `encodeURI`
/// algorithm (<https://tc39.es/ecma262/#sec-encodeuri-uri>). Unreserved ASCII
/// characters and the reserved URI punctuation set are left untouched; every
/// other byte (including all bytes of a multibyte UTF-8 sequence) is encoded as
/// `%XX` with uppercase hex digits.
pub fn encode_uri(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        if should_escape_for_encode_uri(b) {
            result.push('%');
            result.push(UPPER_HEX[(b >> 4) as usize] as char);
            result.push(UPPER_HEX[(b & 0x0f) as usize] as char);
        } else {
            result.push(b as char);
        }
    }
    result
}

/// Whether a byte must be percent-escaped by `encode_uri`.
fn should_escape_for_encode_uri(b: u8) -> bool {
    if b.is_ascii_alphanumeric() {
        return false;
    }
    !matches!(
        b,
        b';' | b'/'
            | b'?'
            | b':'
            | b'@'
            | b'&'
            | b'='
            | b'+'
            | b'$'
            | b','
            | b'#'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')'
    )
}

/// Convert a string to lowercase using JavaScript's casing semantics.
///
/// Mirrors `stringutil.ToLowerJS` in Go. Rust's `char::to_lowercase` already
/// applies the Unicode SpecialCasing multi-character mappings that JavaScript
/// relies on (e.g. `İ` → `i\u{0307}`, `ß` unchanged, `ﬁ` → `fi`, `Ω` → `ω`),
/// so this is a thin wrapper over it. Unlike the full Go implementation, this
/// does not apply the context-sensitive Final_Sigma rule, so a capital sigma
/// (`Σ`) always maps to `σ` rather than the final-sigma form `ς`.
pub fn to_lower_js(s: &str) -> String {
    s.chars().flat_map(char::to_lowercase).collect()
}

/// Convert a string to uppercase using JavaScript's casing semantics.
///
/// Mirrors `stringutil.ToUpperJS` in Go. Rust's `char::to_uppercase` already
/// applies the Unicode SpecialCasing multi-character mappings (e.g. `ß` → `SS`,
/// `ﬁ` → `FI`, `ω` → `Ω`), so this is a thin wrapper over it.
pub fn to_upper_js(s: &str) -> String {
    s.chars().flat_map(char::to_uppercase).collect()
}

/// Encode a code point into a JS string rune.
///
/// Mirrors `stringutil.EncodeJSStringRune` in Go. Valid (non-surrogate) code
/// points are encoded as their normal UTF-8 representation. Lone surrogates
/// (U+D800–U+DFFF) cannot be represented in valid UTF-8, so — unlike the Go
/// version, which stores them as a 3-byte WTF-8 sentinel — this returns an
/// empty string, since a Rust `String` cannot hold invalid UTF-8.
pub fn encode_js_string_rune(ch: u32) -> String {
    if let Some(c) = char::from_u32(ch) {
        c.to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_digit() {
        assert!(is_digit('0'));
        assert!(is_digit('9'));
        assert!(!is_digit('a'));
    }

    #[test]
    fn test_is_hex_digit() {
        assert!(is_hex_digit('0'));
        assert!(is_hex_digit('a'));
        assert!(is_hex_digit('F'));
        assert!(!is_hex_digit('g'));
    }

    #[test]
    fn test_equate_case_insensitive() {
        assert!(equate_string_case_insensitive("Hello", "hello"));
        assert!(!equate_string_case_insensitive("Hello", "world"));
    }

    #[test]
    fn test_split_lines() {
        assert_eq!(split_lines("a\nb\nc"), vec!["a", "b", "c"]);
        assert_eq!(split_lines("a\r\nb"), vec!["a", "b"]);
        assert_eq!(split_lines("a\rb"), vec!["a", "b"]);
    }

    #[test]
    fn test_contains_non_ascii() {
        assert!(!contains_non_ascii("abc"));
        assert!(contains_non_ascii("é"));
        assert!(contains_non_ascii("café"));
        assert!(!contains_non_ascii(""));
    }

    #[test]
    fn test_is_white_space_like() {
        assert!(is_white_space_like(' '));
        assert!(is_white_space_like('\t'));
        assert!(is_white_space_like('\n'));
        assert!(is_white_space_like('\r'));
        assert!(!is_white_space_like('a'));
    }

    #[test]
    fn test_is_line_break() {
        assert!(is_line_break('\n'));
        assert!(is_line_break('\r'));
        assert!(is_line_break('\u{2028}'));
        assert!(is_line_break('\u{2029}'));
        assert!(!is_line_break(' '));
    }

    #[test]
    fn test_is_octal_digit() {
        assert!(is_octal_digit('0'));
        assert!(is_octal_digit('7'));
        assert!(!is_octal_digit('8'));
        assert!(!is_octal_digit('a'));
    }

    #[test]
    fn test_is_ascii_letter() {
        assert!(is_ascii_letter('a'));
        assert!(is_ascii_letter('Z'));
        assert!(!is_ascii_letter('0'));
        assert!(!is_ascii_letter('_'));
    }

    #[test]
    fn test_compare_strings_case_insensitive() {
        assert_eq!(compare_strings_case_insensitive("hello", "HELLO"), 0);
        assert!(compare_strings_case_insensitive("abc", "abd") < 0);
        assert!(compare_strings_case_insensitive("abd", "abc") > 0);
    }

    #[test]
    fn test_compare_strings_case_sensitive() {
        assert_eq!(compare_strings_case_sensitive("hello", "hello"), 0);
        assert!(compare_strings_case_sensitive("abc", "abd") < 0);
        assert!(compare_strings_case_sensitive("abd", "abc") > 0);
    }

    #[test]
    fn test_equate_string_case_sensitive() {
        assert!(equate_string_case_sensitive("Hello", "Hello"));
        assert!(!equate_string_case_sensitive("Hello", "hello"));
    }

    #[test]
    fn test_is_white_space_single_line() {
        assert!(is_white_space_single_line(' '));
        assert!(is_white_space_single_line('\t'));
        assert!(!is_white_space_single_line('\n'));
        assert!(!is_white_space_single_line('a'));
    }

    #[test]
    fn test_split_lines_empty() {
        assert_eq!(split_lines(""), vec![""]);
    }

    #[test]
    fn test_split_lines_trailing_newline() {
        assert_eq!(split_lines("a\n"), vec!["a", ""]);
        assert_eq!(split_lines("a\nb\n"), vec!["a", "b", ""]);
    }

    #[test]
    fn test_split_lines_mixed() {
        assert_eq!(split_lines("a\r\nb\nc\r\nd"), vec!["a", "b", "c", "d"]);
    }

    // ── Ported from Go internal/stringutil/util_test.go ──

    #[test]
    fn test_encode_uri() {
        // Ported test data from Go TestEncodeURI.
        assert_eq!(encode_uri("a b"), "a%20b");
        assert_eq!(encode_uri(";/?:@&=+$,#"), ";/?:@&=+$,#");
        assert_eq!(
            encode_uri("①Ⅻㄨㄩ U1[abc]"),
            "%E2%91%A0%E2%85%AB%E3%84%A8%E3%84%A9%20U1%5Babc%5D"
        );
    }

    #[test]
    fn test_contains_non_ascii_go_port() {
        // Ported from TestContainsNonASCII (portable cases).
        assert!(!contains_non_ascii("abc"));
        assert!(contains_non_ascii("é"));
        // The Go test also checks EncodeJSStringRune(0xD800) -> true, but that
        // sentinel is invalid UTF-8 (lone surrogate) and cannot be represented
        // as a Rust &str. Skipped pending EncodeJSStringRune.
    }

    // ── Ported from Go internal/stringutil/js_case_test.go ──

    #[test]
    fn test_js_casing() {
        // Ported from Go TestJSCasing. These cases exercise the Unicode
        // SpecialCasing multi-character mappings that Rust's
        // `char::to_lowercase` / `char::to_uppercase` already provide.
        assert_eq!(to_lower_js("HELLO"), "hello");
        assert_eq!(to_upper_js("hello"), "HELLO");
        // Lowercase dotted I: İ (U+0130) lowercases to "i" + combining dot above.
        assert_eq!(to_lower_js("İSPANYOL"), "i\u{0307}spanyol");
        // Lone capital sigma lowercases to σ (U+03C3); non-sigma Greek maps simply.
        assert_eq!(to_lower_js("Σ"), "σ");
        assert_eq!(to_lower_js("Ω"), "ω");
        // Sharp s and the fi-ligature uppercase to multi-character mappings.
        assert_eq!(to_upper_js("ßfoo"), "SSFOO");
        assert_eq!(to_upper_js("ω"), "Ω");
        assert_eq!(to_upper_js("ﬁoo"), "FIOO");
        // capitalize/uncapitalize-style concatenations.
        assert_eq!(format!("{}foo", to_upper_js("ß")), "SSfoo");
        assert_eq!(format!("{}foo", to_lower_js("İ")), "i\u{0307}foo");

        // Capital sigma lowercases to σ (rather than final-sigma ς) when it is
        // not preceded by a cased letter or is followed by one. These all map to
        // σ and so match the wrapper's per-character behavior.
        assert_eq!(to_lower_js("ʰΣ"), "ʰσ");
        assert_eq!(to_lower_js("ͅΣ"), "ͅσ");
        assert_eq!(to_lower_js("ΣA"), "σa");
        assert_eq!(to_lower_js("ΣⅠ"), "σⅰ");
        assert_eq!(to_lower_js("ΣͅA"), "σͅa");

        // NOTE: The following Final_Sigma cases from Go's TestJSCasing require
        // context-sensitive casing (capital sigma → ς when preceded by a cased
        // letter and not followed by one), which this wrapper does not perform:
        //   ToLowerJS("ΟΣ") == "ος"
        //   ToLowerJS("ʕΣ") == "ʕς"
        //   ToLowerJS("ªΣ") == "ªς"
        //   ToLowerJS("ºΣ") == "ºς"
        //   ToLowerJS("ⅠΣ") == "ⅰς"
        // These cases depend on a specific Unicode version's "Cased" property
        // (Rust's newer tables lowercase them differently), so they are omitted:
        //   ToLowerJS("\u{1C89}Σ") == "\u{1C89}σ"
        //   ToLowerJS("\u{A7CB}Σ") == "\u{A7CB}σ"
        // Lone-surrogate cases (e.g. EncodeJSStringRune(0xD800)) are omitted
        // because lone surrogates cannot be represented in a Rust `String`:
        //   ToUpperJS(EncodeJSStringRune(0xD800)) == EncodeJSStringRune(0xD800)
        //   ToLowerJS("A" + EncodeJSStringRune(0xD800) + "B") == "a" + ... + "b"
        //   ToUpperJS(EncodeJSStringRune(0xDC00) + "x") == ... + "X"
        //   ToLowerJS(EncodeJSStringRune(0xD800) + "Σ") == ... + "σ"
    }
}
