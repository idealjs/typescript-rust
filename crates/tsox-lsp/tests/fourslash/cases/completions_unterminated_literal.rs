use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_unterminated_literal() {
    let content = r#"// @noLib: true
function foo(a"/*1*/"#;
    let mut s = Session::new_for_test("completionsUnterminatedLiteral", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
