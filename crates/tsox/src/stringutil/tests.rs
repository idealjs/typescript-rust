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

#[test]
fn test_encode_uri() {
    assert_eq!(encode_uri("a b"), "a%20b");
    assert_eq!(encode_uri(";/?:@&=+$,#"), ";/?:@&=+$,#");
    assert_eq!(
        encode_uri("①Ⅻㄨㄩ U1[abc]"),
        "%E2%91%A0%E2%85%AB%E3%84%A8%E3%84%A9%20U1%5Babc%5D"
    );
}

#[test]
fn test_contains_non_ascii_go_port() {
    assert!(!contains_non_ascii("abc"));
    assert!(contains_non_ascii("é"));
}

#[test]
fn test_js_casing() {
    assert_eq!(to_lower_js("HELLO"), "hello");
    assert_eq!(to_upper_js("hello"), "HELLO");

    assert_eq!(to_lower_js("İSPANYOL"), "i\u{0307}spanyol");

    assert_eq!(to_lower_js("Σ"), "σ");
    assert_eq!(to_lower_js("Ω"), "ω");

    assert_eq!(to_upper_js("ßfoo"), "SSFOO");
    assert_eq!(to_upper_js("ω"), "Ω");
    assert_eq!(to_upper_js("ﬁoo"), "FIOO");

    assert_eq!(format!("{}foo", to_upper_js("ß")), "SSfoo");
    assert_eq!(format!("{}foo", to_lower_js("İ")), "i\u{0307}foo");

    assert_eq!(to_lower_js("ʰΣ"), "ʰσ");
    assert_eq!(to_lower_js("ͅΣ"), "ͅσ");
    assert_eq!(to_lower_js("ΣA"), "σa");
    assert_eq!(to_lower_js("ΣⅠ"), "σⅰ");
    assert_eq!(to_lower_js("ΣͅA"), "σͅa");
}
