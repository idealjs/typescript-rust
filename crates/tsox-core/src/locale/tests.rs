use super::*;

#[test]
fn parse_valid_locales() {
    assert_eq!(Locale::parse("en-US"), Some(Locale("en-US".to_string())));
    assert_eq!(Locale::parse("zh"), Some(Locale("zh".to_string())));
    assert_eq!(Locale::parse(""), Some(Locale::default_locale()));
}

#[test]
fn parse_invalid_locales() {
    assert_eq!(Locale::parse("en US"), None);
    assert_eq!(Locale::parse("en!US"), None);
}
