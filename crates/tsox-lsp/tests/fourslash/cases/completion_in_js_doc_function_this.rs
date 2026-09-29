use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_js_doc_function_this() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/** @type {function (this: string, string): string} */
var f = function (s) { return this/**/; }"#;
    let mut s = Session::new_for_test("completionInJSDocFunctionThis", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
