use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_completions18() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: file.js
/**
  * @param {number} a
  * @param {string} b
*/
exports.foo = function(a, b) {
	a/*a*/;
	b/*b*/
};"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions18", content);
    fourslash::go_to_marker(&mut s, "a");
    fourslash::insert(&mut s, ".");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "b");
    fourslash::insert(&mut s, ".");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
