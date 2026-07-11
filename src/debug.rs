//! Debug assertions ported from `internal/debug/debug.go`.
//!
//! These helpers panic with descriptive messages when invariants are
//! violated. They mirror the Go `debug` package so that panic messages
//! match across the two implementations.

use std::fmt::Display;

/// Trait for types that can report their syntax kind as a string.
///
/// Mirrors Go's `interface{ KindString() string }`, used by
/// [`fail_bad_syntax_kind`].
pub trait KindString {
    /// Returns the syntax kind string for this value.
    fn kind_string(&self) -> String;
}

/// Mirrors `debug.Fail` in Go.
///
/// Panics with `"Debug failure."` if `reason` is empty, or
/// `"Debug failure. {reason}"` otherwise.
///
/// # Panics
/// Always panics.
pub fn fail(reason: &str) -> ! {
    let msg = if reason.is_empty() {
        "Debug failure.".to_string()
    } else {
        format!("Debug failure. {}", reason)
    };
    panic!("{}", msg)
}

/// Mirrors `debug.FailBadSyntaxKind` in Go.
///
/// Uses [`KindString::kind_string`] for the node description. The
/// `message` defaults to `"Unexpected node."` when `None`.
///
/// # Panics
/// Always panics.
pub fn fail_bad_syntax_kind<T: KindString>(node: &T, message: Option<&str>) -> ! {
    let msg = message.unwrap_or("Unexpected node.");
    fail(&format!(
        "{}\nNode {} was unexpected.",
        msg,
        node.kind_string()
    ))
}

/// Mirrors `debug.AssertNever` in Go.
///
/// Uses the [`Display`] trait for the detail string. In Go the function
/// tries `KindString` first, then `Stringer`, then `%v`; in Rust we
/// consolidate on `Display`, and types with a `KindString` should
/// implement `Display` by delegating to it.
///
/// The `message` defaults to `"Illegal value:"` when `None`.
///
/// # Panics
/// Always panics.
pub fn assert_never<T: Display>(member: &T, message: Option<&str>) -> ! {
    let msg = message.unwrap_or("Illegal value:");
    fail(&format!("{} {}", msg, member))
}

/// Mirrors `debug.Assert` in Go.
///
/// Does nothing if `value` is true. Otherwise panics with
/// `"Debug failure. False expression."` (no message) or
/// `"Debug failure. False expression: {message}"` (with message).
///
/// # Panics
/// Panics if `value` is false.
pub fn assert(value: bool, message: Option<&str>) {
    if value {
        return;
    }
    let msg = match message {
        Some(m) => format!("False expression: {}", m),
        None => "False expression.".to_string(),
    };
    fail(&msg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Formatter;

    // --- Mock types mirroring the Go test helpers ---

    /// Mirrors Go's `mockNode` which implements `KindString()`.
    struct MockNode {
        kind: String,
    }

    impl MockNode {
        fn new(kind: &str) -> Self {
            Self {
                kind: kind.to_string(),
            }
        }
    }

    impl KindString for MockNode {
        fn kind_string(&self) -> String {
            self.kind.clone()
        }
    }

    // `Display` delegates to `kind_string`, mirroring how Go's `AssertNever`
    // picks `KindString` first for types that implement it.
    impl Display for MockNode {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.kind_string())
        }
    }

    /// Mirrors Go's `mockStringer` which implements `String()`.
    struct MockStringer {
        s: String,
    }

    impl MockStringer {
        fn new(s: &str) -> Self {
            Self { s: s.to_string() }
        }
    }

    impl Display for MockStringer {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.s)
        }
    }

    // --- Tests ported from debug_test.go ---

    #[test]
    #[should_panic(expected = "Debug failure.")]
    fn test_fail_empty_reason() {
        fail("");
    }

    #[test]
    #[should_panic(expected = "Debug failure. something went wrong")]
    fn test_fail_with_reason() {
        fail("something went wrong");
    }

    #[test]
    #[should_panic(expected = "Debug failure. Unexpected node.\nNode FooNode was unexpected.")]
    fn test_fail_bad_syntax_kind_no_message() {
        let node = MockNode::new("FooNode");
        fail_bad_syntax_kind(&node, None);
    }

    #[test]
    #[should_panic(expected = "Debug failure. custom message\nNode BarNode was unexpected.")]
    fn test_fail_bad_syntax_kind_with_message() {
        let node = MockNode::new("BarNode");
        fail_bad_syntax_kind(&node, Some("custom message"));
    }

    #[test]
    #[should_panic(expected = "Debug failure. Illegal value: TestNode")]
    fn test_assert_never_default_message() {
        let node = MockNode::new("TestNode");
        assert_never(&node, None);
    }

    #[test]
    #[should_panic(expected = "Debug failure. bad value: TestNode")]
    fn test_assert_never_custom_message() {
        let node = MockNode::new("TestNode");
        assert_never(&node, Some("bad value:"));
    }

    #[test]
    #[should_panic(expected = "Debug failure. Illegal value: hello")]
    fn test_assert_never_stringer() {
        let s = MockStringer::new("hello");
        assert_never(&s, None);
    }

    #[test]
    #[should_panic(expected = "Debug failure. Illegal value: 42")]
    fn test_assert_never_fallback() {
        assert_never(&42i32, None);
    }

    #[test]
    fn test_assert_true() {
        assert(true, None);
    }

    #[test]
    fn test_assert_true_with_message() {
        assert(true, Some("this should not trigger"));
    }

    #[test]
    #[should_panic(expected = "Debug failure. False expression.")]
    fn test_assert_false_no_message() {
        assert(false, None);
    }

    #[test]
    #[should_panic(expected = "Debug failure. False expression: expected x > 0")]
    fn test_assert_false_with_message() {
        assert(false, Some("expected x > 0"));
    }
}
