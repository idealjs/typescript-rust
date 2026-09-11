use tsox_lsp::fourslash::{self, Session};


#[test]
fn type_of_keyword_completion() {
    let content = r#"export type A = typ/**/"#;
    let mut s = Session::new_for_test("typeOfKeywordCompletion", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
