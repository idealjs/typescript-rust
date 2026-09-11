use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_js_doc_trivia() {
    let content = r#"// @noLib: true
/**
 * @type {{
 * 'string-property': boolean;
 */*$*/ identifierProperty: boolean;
 * }}
 */
var someVariable;"#;
    let mut s = Session::new_for_test("completionsJSDocTrivia", content);
    fourslash::go_to_marker(&mut s, "$");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
