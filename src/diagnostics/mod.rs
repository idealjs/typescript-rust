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
#[derive(Debug, Clone, Copy)]
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
/// Mirrors `diagnostics.Format` in Go.
pub fn format_message(text: &str, args: &[&str]) -> String {
    if args.is_empty() {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            let mut num_str = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    num_str.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            if chars.peek() == Some(&'}') {
                chars.next();
                if let Ok(index) = num_str.parse::<usize>() {
                    if index < args.len() {
                        result.push_str(args[index]);
                        continue;
                    }
                }
            }
            result.push('{');
            result.push_str(&num_str);
            if chars.peek() == Some(&'}') {
                result.push('}');
                chars.next();
            }
        } else {
            result.push(c);
        }
    }

    result
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
}
