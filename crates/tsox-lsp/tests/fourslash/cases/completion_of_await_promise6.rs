use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_of_await_promise6() {
    let content = r#"// @lib: es2015
async function foo(x: Promise<string>) {
   [|x./**/|]
}"#;
    let mut s = Session::new_for_test("completionOfAwaitPromise6", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["catch", "then"]);
}
