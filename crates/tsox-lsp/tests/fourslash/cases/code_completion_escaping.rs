use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_completion_escaping() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: a.js
// @allowJs: true
___foo; __foo;/**/"#;
    let mut s = Session::new_for_test("codeCompletionEscaping", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
