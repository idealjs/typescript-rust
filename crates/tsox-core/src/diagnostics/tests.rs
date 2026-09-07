use super::*;

#[test]
fn category_names() {
    assert_eq!(Category::Warning.name(), "warning");
    assert_eq!(Category::Error.name(), "error");
    assert_eq!(Category::Suggestion.name(), "suggestion");
    assert_eq!(Category::Message.name(), "message");
}

#[test]
fn format_no_args() {
    assert_eq!(format_message("hello world", &[]), "hello world");
}

#[test]
fn format_with_args() {
    assert_eq!(format_message("'{0}' expected", &["foo"]), "'foo' expected");
}

#[test]
fn format_multiple_args() {
    assert_eq!(
        format_message("{0} must precede {1}", &["readonly", "public"]),
        "readonly must precede public"
    );
}

#[test]
fn ad_hoc_message() {
    let msg = new_ad_hoc_message("something went wrong");
    assert_eq!(msg.code, -1);
    assert_eq!(msg.category, Category::Error);
    assert_eq!(msg.text, "something went wrong");
}

#[test]
fn format_non_placeholder_braces_left_untouched() {
    assert_eq!(format_message("{abc}", &["x"]), "{abc}");
}

#[test]
#[should_panic(expected = "Invalid formatting placeholder")]
fn format_panics_on_out_of_range_index() {
    format_message("{5}", &["a", "b", "c"]);
}

#[test]
fn test_localize() {
    assert_eq!(IDENTIFIER_EXPECTED.format(&[]), "Identifier expected.");
    assert_eq!(X_0_EXPECTED.format(&[")"]), "')' expected.");
    assert_eq!(
        THE_PARSER_EXPECTED_TO_FIND_A_1_TO_MATCH_THE_0_TOKEN_HERE.format(&["{", "}"]),
        "The parser expected to find a '}' to match the '{' token here."
    );
}

#[test]
fn test_localize_by_key() {
    let id_msg = key_to_message("Identifier_expected_1003").unwrap();
    assert_eq!(id_msg.format(&[]), "Identifier expected.");
    assert_eq!(id_msg.key, "Identifier_expected_1003");

    let paren_msg = key_to_message("_0_expected_1005").unwrap();
    assert_eq!(paren_msg.format(&[")"]), "')' expected.");
    assert_eq!(paren_msg.key, "_0_expected_1005");
}

#[test]
fn test_localize_zh_cn() {
    let locale = Locale::parse("zh-CN").unwrap();
    let localized = IDENTIFIER_EXPECTED.localize(&locale, &[]);
    assert!(!localized.is_empty());
    assert_ne!(localized, IDENTIFIER_EXPECTED.text);
    assert_eq!(localized, "应为标识符。");
}

#[test]
fn test_localize_with_args() {
    let locale = Locale::parse("zh-CN").unwrap();
    let localized = X_0_EXPECTED.localize(&locale, &["x"]);
    assert_eq!(localized, "应为“x”。");
}

#[test]
fn test_localize_falls_back_to_english_for_unknown_locale() {
    let locale = Locale::parse("klingon").unwrap();
    let localized = IDENTIFIER_EXPECTED.localize(&locale, &[]);
    assert_eq!(localized, IDENTIFIER_EXPECTED.text);
}

#[test]
fn test_localize_falls_back_to_english_for_missing_key() {
    let locale = Locale::parse("zh-CN").unwrap();
    let msg = new_ad_hoc_message("ad hoc only");
    let localized = msg.localize(&locale, &[]);
    assert_eq!(localized, "ad hoc only");
}
