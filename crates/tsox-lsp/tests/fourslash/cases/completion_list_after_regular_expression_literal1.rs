use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_regular_expression_literal1() {
    let content = r#"// @lib: es5
/a/./**/"#;
    let mut s = Session::new_for_test("completionListAfterRegularExpressionLiteral1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
