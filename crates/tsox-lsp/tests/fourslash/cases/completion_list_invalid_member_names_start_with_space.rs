use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_invalid_member_names_start_with_space() {
    let content = r#"declare const x: { " foo": 0, "foo ": 1 };
x[|./**/|];"#;
    let mut s = Session::new_for_test("completionListInvalidMemberNames_startWithSpace", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
