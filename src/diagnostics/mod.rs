//! Diagnostic messages and formatting.
//!
//! Ported from `internal/diagnostics/` in the Go implementation.
//! The core types (`Category`, `Message`) are hand-written here;
//! the ~2000 diagnostic message constants are generated in
//! `messages_generated.rs` by `_scripts/generate-rust-diagnostics.ts`.

pub mod messages_generated;

pub use messages_generated::*;

/// Diagnostic severity category.
///
/// Mirrors `diagnostics.Category` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum Category {
    #[default]
    Warning,
    Error,
    Suggestion,
    Message,
}

impl Category {
    pub fn name(self) -> &'static str {
        match self {
            Category::Warning => "warning",
            Category::Error => "error",
            Category::Suggestion => "suggestion",
            Category::Message => "message",
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// A diagnostic message key, used for localization lookup.
///
/// Mirrors `diagnostics.Key` in Go.
pub type Key = &'static str;

/// A localizable diagnostic message.
///
/// Mirrors `diagnostics.Message` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Message {
    pub code: i32,
    pub category: Category,
    pub key: Key,
    pub text: &'static str,
    pub reports_unnecessary: bool,
    pub elided_in_compatibility_pyramid: bool,
    pub reports_deprecated: bool,
}

impl Message {
    pub fn code(&self) -> i32 {
        self.code
    }

    pub fn category(&self) -> Category {
        self.category
    }

    pub fn key(&self) -> Key {
        self.key
    }

    pub fn reports_unnecessary(&self) -> bool {
        self.reports_unnecessary
    }

    pub fn reports_deprecated(&self) -> bool {
        self.reports_deprecated
    }

    /// Format the message text with the given arguments, replacing
    /// `{0}`, `{1}`, etc. placeholders.
    pub fn format(&self, args: &[&str]) -> String {
        format_message(self.text, args)
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.text)
    }
}

/// Create an ad-hoc error message (not in the generated message table).
///
/// Mirrors `diagnostics.NewAdHocMessage` in Go.
pub fn new_ad_hoc_message(text: &'static str) -> Message {
    Message {
        code: -1,
        category: Category::Error,
        key: "-1",
        text,
        reports_unnecessary: false,
        elided_in_compatibility_pyramid: false,
        reports_deprecated: false,
    }
}

/// Format a text template by replacing `{N}` placeholders with args.
///
/// Mirrors `diagnostics.Format` in Go (`diagnostics.go:117-133`):
/// - Returns `text` unchanged when `args` is empty.
/// - Replaces invalid UTF-8 in args with U+FFFD (Rust `&str` is always
///   valid UTF-8, so this is a no-op here, but the sanitization step is
///   documented for parity).
/// - Uses the regex `{(\d+)}` to locate placeholders; for each match,
///   parses the index and panics with `"Invalid formatting placeholder"`
///   when the index is out of range — matching Go's panic on programming
///   errors (a diagnostic text referencing `{N}` must supply enough args).
pub fn format_message(text: &str, args: &[&str]) -> String {
    if args.is_empty() {
        return text.to_string();
    }

    // Rust `&str` is always valid UTF-8, so the `strings.ToValidUTF8`
    // sanitization in Go is a no-op here. Args are used as-is.
    let re = regex::Regex::new(r"\{(\d+)\}").expect("valid regex");
    re.replace_all(text, |caps: &regex::Captures| {
        let index: usize = caps
            .get(1)
            .expect("capture group 1")
            .as_str()
            .parse()
            .expect("Invalid formatting placeholder");
        if index >= args.len() {
            panic!("Invalid formatting placeholder");
        }
        args[index]
    })
    .into_owned()
}

#[cfg(test)]
mod tests {
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
        // `{abc}` is not a valid `{N}` placeholder and is left as-is,
        // matching Go's regex `{(\d+)}` which does not match it.
        assert_eq!(format_message("{abc}", &["x"]), "{abc}");
    }

    #[test]
    #[should_panic(expected = "Invalid formatting placeholder")]
    fn format_panics_on_out_of_range_index() {
        // Go panics when a `{N}` placeholder references an arg index that
        // does not exist; Rust aligns with this behavior.
        format_message("{5}", &["a", "b", "c"]);
    }

    // ────────────────────────────────────────────────────────────────────
    // Ported from Go internal/diagnostics/diagnostics_test.go
    // ────────────────────────────────────────────────────────────────────

    #[test]
    #[ignore = "TODO: localization (Message::localize) is not implemented in Rust"]
    fn test_localize() {
        // Ported 1:1 from Go TestLocalize.
        //
        // Go's `Message::Localize(locale, args...)` looks up a localized
        // template for the given BCP-47 locale (falling back to English for an
        // undefined or unknown locale) and formats it with the args. Rust has
        // no locale message catalog — the `diagnostics/loc/*.json.gz` resources
        // and the `Message::localize` / `locale::Locale(language.Tag)` APIs are
        // not ported. `Message::format` only ever produces the English text, so
        // the non-English expectations below cannot be satisfied yet.
        //
        // Expected results (message, locale, args, expected):
        //   Identifier_expected, English,         []          -> "Identifier expected."
        //   Identifier_expected, Und,             []          -> "Identifier expected."
        //   X_0_expected,       English,          [")"]       -> "')' expected."
        //   ..._0_token_here,   English,          ["{", "}"]  -> "The parser expected to find a '}' to match the '{' token here."
        //   Identifier_expected, af-ZA (unknown), []          -> "Identifier expected."  (fallback to English)
        //   Identifier_expected, de-DE,           []          -> "Es wurde ein Bezeichner erwartet."
        //   Identifier_expected, fr-FR,           []          -> "Identificateur attendu."
        //   Identifier_expected, es-ES,           []          -> "Se esperaba un identificador."
        //   Identifier_expected, ja-JP,           []          -> "識別子が必要です。"
        //   Identifier_expected, zh-CN,           []          -> "应为标识符。"
        //   Identifier_expected, ko-KR,           []          -> "식별자가 필요합니다."
        //   Identifier_expected, ru-RU,           []          -> "Ожидался идентификатор."
        //   X_0_expected,       de-DE,            [")"]       -> "\")\" wurde erwartet."
        //
        // TODO: once localization lands, build the table above and assert:
        //   assert_eq!(message.localize(&locale, &args), expected);
        let _ = (
            IDENTIFIER_EXPECTED,
            X_0_EXPECTED,
            THE_PARSER_EXPECTED_TO_FIND_A_1_TO_MATCH_THE_0_TOKEN_HERE,
        );
    }

    #[test]
    #[ignore = "TODO: localization (free function localize by key) is not implemented in Rust"]
    fn test_localize_by_key() {
        // Ported 1:1 from Go TestLocalize_ByKey.
        //
        // Go's free function `Localize(locale, nil, key, args...)` looks up a
        // message by its diagnostic key string (e.g.
        // "Identifier_expected_1003") and localizes it. Rust has no
        // key -> localized-template catalog and no `localize` free function,
        // so this is ignored until localization lands.
        //
        // Expected results (key, locale, args, expected):
        //   "Identifier_expected_1003", English, []    -> "Identifier expected."
        //   "_0_expected_1005",         English, [")"] -> "')' expected."
        //
        // TODO: once localization lands, assert:
        //   assert_eq!(localize(&locale, None, key, &args), expected);
    }
}
