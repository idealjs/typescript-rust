use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("getJavaScriptCompletions5", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
