use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_of_await_promise5() {
    let content = r#"interface Foo { foo: string }
async function foo(x: (a: number) => Promise<Foo>) {
   [|x(1)./**/|]
}"#;
    let mut s = Session::new_for_test("completionOfAwaitPromise5", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
