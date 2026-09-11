use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn get_java_script_completions10() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/**
 * @type {function(this:number)}
 */
function f() { this./**/ }"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions10", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
