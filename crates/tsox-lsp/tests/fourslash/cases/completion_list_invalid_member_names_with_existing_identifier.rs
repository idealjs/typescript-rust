use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_invalid_member_names_with_existing_identifier() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"declare const x: { "foo ": "space in the name", };
x[|.fo/*0*/|];
x[|./*1*/|]
unrelatedIdentifier;"#;
    let mut s = Session::new_for_test("completionListInvalidMemberNames_withExistingIdentifier", content);
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
