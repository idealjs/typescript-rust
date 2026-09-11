use tsox_lsp::fourslash::{self, Session};


#[test]
fn unclosed_string_literal_error_recovery() {
    let content = r#""an unclosed string is a terrible thing!

class foo { public x() { } }
var f = new foo();
f./**/"#;
    let mut s = Session::new_for_test("unclosedStringLiteralErrorRecovery", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["x"]);
}
