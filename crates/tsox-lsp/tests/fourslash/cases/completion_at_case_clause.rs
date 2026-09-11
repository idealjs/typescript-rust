use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_at_case_clause() {
    let content = r#"// @lib: es5
case /**/"#;
    let mut s = Session::new_for_test("completionAtCaseClause", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
