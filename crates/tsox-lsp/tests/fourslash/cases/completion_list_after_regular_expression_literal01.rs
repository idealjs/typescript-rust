use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_regular_expression_literal01() {
    let content = r#"// @lib: es5
let v = 100;
/a/./**/"#;
    let mut s = Session::new_for_test("completionListAfterRegularExpressionLiteral01", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
