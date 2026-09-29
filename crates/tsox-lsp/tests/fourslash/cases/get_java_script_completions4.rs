use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_completions4() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @return {number} */
function foo(a,b) { }
foo(1,2)./**/"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions4", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
