use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn jsdoc_satisfies_tag_completion1() {
    let content = r#"// @lib: es5
// @noEmit: true
// @allowJS: true
// @checkJs: true
// @filename: /a.js
/**
 * @satisfies {/**/}
 */
const t = { a: 1 };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
