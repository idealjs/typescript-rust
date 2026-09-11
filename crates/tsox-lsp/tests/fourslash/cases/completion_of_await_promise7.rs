use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_of_await_promise7() {
    let content = r#"async function foo(x: Promise<string>) {
    console.log
    [|x./**/|]
}"#;
    let mut s = Session::new_for_test("completionOfAwaitPromise7", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
