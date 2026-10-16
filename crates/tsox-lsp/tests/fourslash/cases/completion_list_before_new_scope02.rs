use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_before_new_scope02() {
    let content = r#"a/*1*/

{
    let aaaaaa = 10;
}"#;
    let mut s = Session::new_for_test("completionListBeforeNewScope02", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
