use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_of_await_promise1() {
    let content = r#"async function foo(x: Promise<string>) {
   [|x./**/|]
}"#;
    let mut s = Session::new_for_test("completionOfAwaitPromise1", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
