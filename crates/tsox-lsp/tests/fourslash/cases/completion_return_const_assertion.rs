use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_return_const_assertion() {
    let content = r#"type T = {
    foo1: 1;
    foo2: 2;
}
function F(x: ()=>T) {}
F(()=>({/*1*/} as const))"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
