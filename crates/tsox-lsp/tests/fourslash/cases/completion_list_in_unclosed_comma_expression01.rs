use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_unclosed_comma_expression01() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// should NOT see a and b
foo((a, b) => a,/*1*/"#;
    let mut s = Session::new_for_test("completionListInUnclosedCommaExpression01", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
