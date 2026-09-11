use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_completion_escaping() {
    let content = r#"// @Filename: a.js
// @allowJs: true
___foo; __foo;/**/"#;
    let mut s = Session::new_for_test("codeCompletionEscaping", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
