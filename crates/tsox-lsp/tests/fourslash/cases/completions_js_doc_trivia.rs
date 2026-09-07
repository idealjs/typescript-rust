use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "$");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
