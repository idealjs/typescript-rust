use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_named_function_expression_with_shadowing() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function foo() {}
/*0*/
var x = function foo() {
   /*1*/
}
var y = function () {
   /*2*/
}"#;
    let mut s = Session::new_for_test("completionListInNamedFunctionExpressionWithShadowing", content);
    // TODO: f.VerifyCompletions(t, []string{"0", "2"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
