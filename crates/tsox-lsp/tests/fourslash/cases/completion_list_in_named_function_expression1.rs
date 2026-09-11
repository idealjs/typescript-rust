use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_named_function_expression1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"var x = function foo() {
   /*1*/
}"#;
    let mut s = Session::new_for_test("completionListInNamedFunctionExpression1", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
