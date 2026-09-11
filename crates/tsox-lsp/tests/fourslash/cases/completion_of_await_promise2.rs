use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_of_await_promise2() {
    let content = r#"interface Foo { foo: string }
async function foo(x: Promise<Foo>) {
   [|x./**/|]
}"#;
    let mut s = Session::new_for_test("completionOfAwaitPromise2", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
