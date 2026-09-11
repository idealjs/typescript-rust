use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_in_function_call2() {
    let content = r#"type T = {
    a: 1;
    b: 2;
}
function F(x: T) {
}
F({/*1*/} as const)"#;
    let mut s = Session::new_for_test("memberListInFunctionCall2", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
