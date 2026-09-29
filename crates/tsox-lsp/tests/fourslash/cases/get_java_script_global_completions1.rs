use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn get_java_script_global_completions1() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
function f() {
    // helloWorld leaks from here into the global space?
    if (helloWorld) {
        return 3;
    }
    return 5;
}

hello/**/"#;
    let _s = Session::new_for_test("getJavaScriptGlobalCompletions1", content);
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
