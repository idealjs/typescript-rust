use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_of_await_promise2() {
    let content = r#"interface Foo { foo: string }
async function foo(x: Promise<Foo>) {
   [|x./**/|]
}"#;
    let mut s = Session::new_for_test("completionOfAwaitPromise2", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
