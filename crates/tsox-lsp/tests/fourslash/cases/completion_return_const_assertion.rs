use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_return_const_assertion() {
    let content = r#"type T = {
    foo1: 1;
    foo2: 2;
}
function F(x: ()=>T) {}
F(()=>({/*1*/} as const))"#;
    let mut s = Session::new_for_test("completionReturnConstAssertion", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
