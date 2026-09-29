use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_completions1() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @type {number} */
var v;
v./**/"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
