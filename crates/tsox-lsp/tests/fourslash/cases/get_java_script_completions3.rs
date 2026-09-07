use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn get_java_script_completions3() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @type {Array.<number>} */
var v;
v./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
