use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_completions13() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowNonTsExtensions: true
// @Filename: file1.js
var file1Identifier = 1;
interface Foo { FooProp: number };
// @Filename: file2.js
var file2Identifier1 = 2;
var file2Identifier2 = 2;
/*1*/
file2Identifier2./*2*/"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions13", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
