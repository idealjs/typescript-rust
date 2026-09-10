use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_at_case_clause() {
    let content = r#"// @lib: es5
case /**/"#;
    let mut s = Session::new_for_test("completionAtCaseClause", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
