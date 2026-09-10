use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn string_completion_details() {
    let content = r#"const a: "aa" | "bb" = "/**/";"#;
    let mut s = Session::new_for_test("stringCompletionDetails", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
