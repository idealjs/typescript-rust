use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_of_await_promise3() {
    let content = r#"interface Foo { ["foo-foo"]: string }
async function foo(x: Promise<Foo>) {
   [|x./**/|]
}"#;
    let mut s = Session::new_for_test("completionOfAwaitPromise3", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
