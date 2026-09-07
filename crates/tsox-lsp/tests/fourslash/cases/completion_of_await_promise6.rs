use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_of_await_promise6() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es2015
async function foo(x: Promise<string>) {
   [|x./**/|]
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
