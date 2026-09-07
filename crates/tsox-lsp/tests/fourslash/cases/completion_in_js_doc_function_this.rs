use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_in_js_doc_function_this() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/** @type {function (this: string, string): string} */
var f = function (s) { return this/**/; }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
