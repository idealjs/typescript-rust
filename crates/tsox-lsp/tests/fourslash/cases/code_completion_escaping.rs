use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_completion_escaping() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: a.js
// @allowJs: true
___foo; __foo;/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
