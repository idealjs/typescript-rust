use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn get_java_script_completions5() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/**
 * @template T
 * @param {T} a
 * @return {T} */
function foo(a) { }
let x = foo;
foo(1)./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
