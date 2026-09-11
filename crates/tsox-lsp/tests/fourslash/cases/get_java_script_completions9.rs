use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn get_java_script_completions9() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/**
 * @type {function(new:number)}
 */
var v;
new v()./**/"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions9", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
