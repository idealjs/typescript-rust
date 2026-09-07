use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_closed_function03() {
    let content = r#"function foo(x: string, y: number, z: boolean) {
    function bar(a: number, b: string, c: typeof x = /*1*/) {

    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
