use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn unclosed_string_literal_error_recovery() {
    let content = r#""an unclosed string is a terrible thing!

class foo { public x() { } }
var f = new foo();
f./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
