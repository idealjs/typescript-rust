use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_keywords() {
    let content = r#"// @noLib: true
/**/"#;
    let mut s = Session::new_for_test("completionListKeywords", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
