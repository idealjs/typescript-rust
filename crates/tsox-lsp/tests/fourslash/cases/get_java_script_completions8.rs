use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn get_java_script_completions8() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/**
 * @type {function(): number}
 */
var v;
v()./**/"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions8", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
