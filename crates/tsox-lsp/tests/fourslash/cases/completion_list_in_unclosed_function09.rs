use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_in_unclosed_function09() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function foo(x: string, y: number, z: boolean) {
    function bar(a: number, b: string = "hello", c: typeof x = "hello") {
        var v = /*1*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: }
}
