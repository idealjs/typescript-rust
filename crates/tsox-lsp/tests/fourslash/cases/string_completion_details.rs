use tsox_lsp::fourslash::{self, Session};


#[test]
fn string_completion_details() {
    let content = r#"const a: "aa" | "bb" = "/**/";"#;
    let mut s = Session::new_for_test("stringCompletionDetails", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
