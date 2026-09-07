use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn completion_list_in_unclosed_function06() {
    let content = r#"function foo(x: string, y: number, z: boolean) {
    function bar(a: number, b: string = /*1*/, c: typeof x = "hello"
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: }
}
