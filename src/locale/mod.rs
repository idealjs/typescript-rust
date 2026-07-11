//! Locale handling, ported from `internal/locale/`.
//!
//! In the Go version, locale is propagated via context. In Rust, we use a
//! simple struct that can be passed explicitly.

use std::fmt;

/// A locale tag (e.g., "en-US", "zh-CN").
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Locale(pub String);

impl Locale {
    /// The default locale (empty string = system default).
    pub fn default_locale() -> Locale {
        Locale(String::new())
    }

    /// Parse a locale string. Returns `None` if the string is not a valid locale.
    pub fn parse(s: &str) -> Option<Locale> {
        if s.is_empty() {
            return Some(Locale::default_locale());
        }
        // Basic validation: must be ASCII alphanumeric with hyphens
        if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            Some(Locale(s.to_string()))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Locale {
    fn from(s: &str) -> Self {
        Locale(s.to_string())
    }
}

impl From<String> for Locale {
    fn from(s: String) -> Self {
        Locale(s)
    }
}

#[cfg(test)]
mod tests {
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
}
