use tsox_lsp::fourslash::{self, Session};


#[test]
fn extends_keyword_completion1() {
    let content = r#"export interface B ex/**/"#;
    let mut s = Session::new_for_test("extendsKeywordCompletion1", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
