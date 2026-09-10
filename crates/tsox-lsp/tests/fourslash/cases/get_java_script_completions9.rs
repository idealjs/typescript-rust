use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn get_java_script_completions9() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/**
 * @type {function(new:number)}
 */
var v;
new v()./**/"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions9", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
