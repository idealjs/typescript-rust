use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_after_backslash_following_string() {
    let content = r#"// @lib: es5
Harness.newLine = ""\n/**/"#;
    let mut s = Session::new_for_test("completionAfterBackslashFollowingString", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
