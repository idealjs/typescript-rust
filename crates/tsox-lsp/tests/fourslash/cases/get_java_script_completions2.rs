use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn get_java_script_completions2() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @type {(number|string)} */
var v;
v./**/"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions2", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
