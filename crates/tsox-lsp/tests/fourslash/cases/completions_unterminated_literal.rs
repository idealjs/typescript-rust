use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_unterminated_literal() {
    let content = r#"// @noLib: true
function foo(a"/*1*/"#;
    let mut s = Session::new_for_test("completionsUnterminatedLiteral", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
