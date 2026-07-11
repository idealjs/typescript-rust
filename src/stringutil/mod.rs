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
}
