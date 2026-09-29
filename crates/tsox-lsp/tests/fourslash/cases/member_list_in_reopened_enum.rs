use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn member_list_in_reopened_enum() {
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
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
