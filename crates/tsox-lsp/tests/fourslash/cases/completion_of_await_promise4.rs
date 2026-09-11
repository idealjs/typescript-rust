use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_of_await_promise4() {
    let content = r#"function foo(x: Promise<string>) {
   [|x./**/|]
}"#;
    let mut s = Session::new_for_test("completionOfAwaitPromise4", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
