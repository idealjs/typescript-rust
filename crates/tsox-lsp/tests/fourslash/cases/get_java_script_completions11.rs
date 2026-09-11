use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_completions11() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @type {number|string} */
var v;
v./**/"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions11", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
