use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_in_named_function_expression1() {
    let content = r#"var x = function foo() {
   /*1*/
}"#;
    let mut s = Session::new_for_test("completionListInNamedFunctionExpression1", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
