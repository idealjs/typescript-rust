use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_invalid_member_names_with_existing_identifier() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"declare const x: { "foo ": "space in the name", };
x[|.fo/*0*/|];
x[|./*1*/|]
unrelatedIdentifier;"#;
    let mut s = Session::new_for_test("completionListInvalidMemberNames_withExistingIdentifier", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
