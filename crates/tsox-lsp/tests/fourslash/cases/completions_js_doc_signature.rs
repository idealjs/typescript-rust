use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_js_doc_signature() {
    let content = r#"// @noLib: true
// @checkJs: true
// @allowJs: true
// @filename: index.js
/**
 * @type {{
 *   (input: string):/*1*/ X|Y/*2*/
 * }}
 */
let x;"#;
    let mut s = Session::new_for_test("completionsJSDocSignature", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
