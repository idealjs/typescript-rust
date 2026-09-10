use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_in_js_doc_property_with_link_no_crash1() {
    let content = r#"
// @allowJs: true
// @filename: /file.js
export function foo() {}

/**
 * @typedef MyType
 * @property {number} [timeout] - The /*1*/timeout; defaults to {@linkcode DEFAULT}
 */
"#;
    let mut s = Session::new_for_test("completionInJSDocPropertyWithLinkNoCrash1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
