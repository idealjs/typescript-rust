use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_before_new_scope01() {
    let content = r#"p/*1*/

function fun(param) {
    let party = Math.random() < 0.99;
}"#;
    let mut s = Session::new_for_test("completionListBeforeNewScope01", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
