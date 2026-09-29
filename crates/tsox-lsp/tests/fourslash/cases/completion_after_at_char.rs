use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_after_at_char() {
    let content = r#"// @lib: es5
@a/**/"#;
    let mut s = Session::new_for_test("completionAfterAtChar", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
