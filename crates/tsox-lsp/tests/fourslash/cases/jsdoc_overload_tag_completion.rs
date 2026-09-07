use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn jsdoc_overload_tag_completion() {
    let content = r#"// @allowJS: true
// @checkJs: true
// @filename: /a.js
/**
 * @/**/
 */"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
