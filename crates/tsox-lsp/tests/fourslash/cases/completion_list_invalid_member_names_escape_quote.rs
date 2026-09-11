use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_invalid_member_names_escape_quote() {
    let content = r#"declare const x: { "\"'": 0 };
x[|./**/|];"#;
    let mut s = Session::new_for_test("completionListInvalidMemberNames_escapeQuote", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
