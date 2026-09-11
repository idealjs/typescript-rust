use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_js_doc_function_new() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/** @type {function (new: string, string): string} */
var f = function () { return new/**/; }"#;
    let mut s = Session::new_for_test("completionInJSDocFunctionNew", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
