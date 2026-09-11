use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_in_reopened_enum() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"namespace M {
    enum E {
        A, B
    }
    enum E {
        C = 0, D
    }
    var x = E./*1*/
}"#;
    let mut s = Session::new_for_test("memberListInReopenedEnum", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
