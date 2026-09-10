use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn extends_keyword_completion1() {
    let content = r#"export interface B ex/**/"#;
    let mut s = Session::new_for_test("extendsKeywordCompletion1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
