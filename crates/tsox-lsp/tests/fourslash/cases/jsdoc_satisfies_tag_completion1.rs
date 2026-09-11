use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("jsdocSatisfiesTagCompletion1", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
