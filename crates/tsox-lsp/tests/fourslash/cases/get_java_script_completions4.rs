use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn get_java_script_completions4() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/** @return {number} */
function foo(a,b) { }
foo(1,2)./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
