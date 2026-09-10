use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_unterminated_literal() {
    let content = r#"// @noLib: true
function foo(a"/*1*/"#;
    let mut s = Session::new_for_test("completionsUnterminatedLiteral", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
