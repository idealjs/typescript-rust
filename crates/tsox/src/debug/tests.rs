use super::*;
use std::fmt::Formatter;

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

impl Display for MockNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind_string())
    }
}

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
