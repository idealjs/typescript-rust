use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_in_unclosed_comma_expression01() {
    let content = r#"// should NOT see a and b
foo((a, b) => a,/*1*/"#;
    let mut s = Session::new_for_test("completionListInUnclosedCommaExpression01", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
