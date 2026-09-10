use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_of_await_promise4() {
    let content = r#"function foo(x: Promise<string>) {
   [|x./**/|]
}"#;
    let mut s = Session::new_for_test("completionOfAwaitPromise4", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
