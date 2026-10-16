use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_after_dot_dot_dot() {
    let content = r#"// @lib: es5
.../**/"#;
    let mut s = Session::new_for_test("completionAfterDotDotDot", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
