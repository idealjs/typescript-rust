//! Diagnostic messages and formatting.
//!
//! Ported from `internal/diagnostics/` in the Go implementation.
//! The core types (`Category`, `Message`) are hand-written here;
//! the ~2000 diagnostic message constants are generated in
//! `messages_generated.rs` by `_scripts/generate-rust-diagnostics.ts`.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::locale::Locale;

pub mod messages_generated;

// ────────────────────────────────────────────────────────────────────────────
// Localization
// ────────────────────────────────────────────────────────────────────────────

/// All embedded locale bundles, lazily decompressed on first access.
///
/// Each entry maps a BCP-47 locale tag (e.g. "zh-CN") to a map of diagnostic
/// message key → translated template. The bundles are embedded at compile time
/// via `include_bytes!` and gzip-decompressed on demand, mirroring the Go
/// implementation's `diagnostics/generateLocals` / `diagnostics/localizeFromMap`.
static LOCALE_MAPS: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();

/// Decompress a gzip-encoded byte slice, returning an empty `Vec` on failure.
fn decompress_gzip(data: &[u8]) -> Vec<u8> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    let mut decoder = GzDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result).unwrap_or_default();
    result
}

/// Load and decompress all embedded locale bundles into a single map.
///
/// The 13 locale files shipped with the Go compiler (`*.json.gz`) are embedded
/// verbatim; each is gzip-decoded and parsed as `{key: text}` JSON. A malformed
/// bundle is silently replaced with an empty map so a single corrupt file can
/// never prevent compilation (the English fallback on `Message` covers missing
/// keys).
fn load_locale_map() -> HashMap<&'static str, HashMap<String, String>> {
    let mut maps = HashMap::new();
    for (tag, raw) in [
        ("cs-CZ", include_bytes!("loc/cs-CZ.json.gz").as_slice()),
        ("de-DE", include_bytes!("loc/de-DE.json.gz").as_slice()),
        ("es-ES", include_bytes!("loc/es-ES.json.gz").as_slice()),
        ("fr-FR", include_bytes!("loc/fr-FR.json.gz").as_slice()),
        ("it-IT", include_bytes!("loc/it-IT.json.gz").as_slice()),
        ("ja-JP", include_bytes!("loc/ja-JP.json.gz").as_slice()),
        ("ko-KR", include_bytes!("loc/ko-KR.json.gz").as_slice()),
        ("pl-PL", include_bytes!("loc/pl-PL.json.gz").as_slice()),
        ("pt-BR", include_bytes!("loc/pt-BR.json.gz").as_slice()),
        ("ru-RU", include_bytes!("loc/ru-RU.json.gz").as_slice()),
        ("tr-TR", include_bytes!("loc/tr-TR.json.gz").as_slice()),
        ("zh-CN", include_bytes!("loc/zh-CN.json.gz").as_slice()),
        ("zh-TW", include_bytes!("loc/zh-TW.json.gz").as_slice()),
    ] {
        let decompressed = decompress_gzip(raw);
        let map: HashMap<String, String> =
            serde_json::from_slice(&decompressed).unwrap_or_default();
        maps.insert(tag, map);
    }
    maps
}

/// Return the shared locale-bundle table, initializing it on first call.
fn locale_maps() -> &'static HashMap<&'static str, HashMap<String, String>> {
    LOCALE_MAPS.get_or_init(load_locale_map)
}

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

    /// Format the message for `locale`, falling back to the English text when no
    /// translation is available.
    ///
    /// Mirrors Go's `Message.Localize`: look up the localized template by the
    /// locale's BCP-47 tag, then by its language prefix (e.g. "zh" for "zh-CN"),
    /// and finally fall back to the compiled-in English `text`. The resulting
    /// template is formatted with `args` using the same `{N}` placeholder rules
    /// as [`Message::format`].
    pub fn localize(&self, locale: &Locale, args: &[&str]) -> String {
        let locale_str = locale.as_str();
        let maps = locale_maps();
        let text = maps
            .get(locale_str)
            .and_then(|m| m.get(self.key))
            .or_else(|| {
                if let Some(lang) = locale_str.split('-').next() {
                    maps.get(lang).and_then(|m| m.get(self.key))
                } else {
                    None
                }
            })
            .map(String::as_str)
            .unwrap_or(self.text);
        format_message(text, args)
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
    fn test_localize() {
        // Ported from Go TestLocalize, adapted to the Rust English fallback.
        //
        // Go's `Message::Localize(locale, args...)` looks up a localized
        // template for the given BCP-47 locale (falling back to English for an
        // undefined or unknown locale) and formats it with the args. Rust has
        // no locale message catalog yet — `Message::format` only ever produces
        // the English text. We assert the English fallback (which is also what
        // Go returns for `Und`/unknown locales) for the representative cases.
        assert_eq!(IDENTIFIER_EXPECTED.format(&[]), "Identifier expected.");
        assert_eq!(X_0_EXPECTED.format(&[")"]), "')' expected.");
        assert_eq!(
            THE_PARSER_EXPECTED_TO_FIND_A_1_TO_MATCH_THE_0_TOKEN_HERE.format(&["{", "}"]),
            "The parser expected to find a '}' to match the '{' token here."
        );
    }

    #[test]
    fn test_localize_by_key() {
        // Ported from Go TestLocalize_ByKey, adapted to the Rust English
        // fallback. Go's free function `Localize(locale, nil, key, args...)`
        // looks up a message by its diagnostic key string. Rust has no
        // localized catalog, but `key_to_message` resolves a key to its
        // (English) `Message`, which we then format — matching Go's English
        // output.
        let id_msg = key_to_message("Identifier_expected_1003").unwrap();
        assert_eq!(id_msg.format(&[]), "Identifier expected.");
        assert_eq!(id_msg.key, "Identifier_expected_1003");

        let paren_msg = key_to_message("_0_expected_1005").unwrap();
        assert_eq!(paren_msg.format(&[")"]), "')' expected.");
        assert_eq!(paren_msg.key, "_0_expected_1005");
    }

    #[test]
    fn test_localize_zh_cn() {
        // The zh-CN bundle is loaded from the embedded `loc/zh-CN.json.gz`;
        // `localize` resolves the translated template by diagnostic key and
        // formats it with the supplied args.
        let locale = Locale::parse("zh-CN").unwrap();
        let localized = IDENTIFIER_EXPECTED.localize(&locale, &[]);
        assert!(!localized.is_empty());
        assert_ne!(localized, IDENTIFIER_EXPECTED.text);
        assert_eq!(localized, "应为标识符。");
    }

    #[test]
    fn test_localize_with_args() {
        // A translated template with a `{0}` placeholder is formatted using the
        // locale-appropriate text (zh-CN: '“{0}” expected.' → '"x" expected.').
        let locale = Locale::parse("zh-CN").unwrap();
        let localized = X_0_EXPECTED.localize(&locale, &["x"]);
        assert_eq!(localized, "应为“x”。");
    }

    #[test]
    fn test_localize_falls_back_to_english_for_unknown_locale() {
        // An unrecognized locale has no bundle, so localization falls back to
        // the compiled-in English text.
        let locale = Locale::parse("klingon").unwrap();
        let localized = IDENTIFIER_EXPECTED.localize(&locale, &[]);
        assert_eq!(localized, IDENTIFIER_EXPECTED.text);
    }

    #[test]
    fn test_localize_falls_back_to_english_for_missing_key() {
        // An ad-hoc message with a key absent from the bundle falls back to its
        // own English text.
        let locale = Locale::parse("zh-CN").unwrap();
        let msg = new_ad_hoc_message("ad hoc only");
        let localized = msg.localize(&locale, &[]);
        assert_eq!(localized, "ad hoc only");
    }
}
