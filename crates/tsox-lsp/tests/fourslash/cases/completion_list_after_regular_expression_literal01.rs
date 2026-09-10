use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_after_regular_expression_literal01() {
    let content = r#"// @lib: es5
let v = 100;
/a/./**/"#;
    let mut s = Session::new_for_test("completionListAfterRegularExpressionLiteral01", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
