use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_of_await_promise7() {
    let content = r#"async function foo(x: Promise<string>) {
    console.log
    [|x./**/|]
}"#;
    let mut s = Session::new_for_test("completionOfAwaitPromise7", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
