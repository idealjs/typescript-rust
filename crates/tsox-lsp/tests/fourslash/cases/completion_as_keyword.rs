use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_as_keyword() {
    let content = r#"const x = this /*1*/
function foo() {
    const x = this /*2*/
}"#;
    let mut s = Session::new_for_test("completionAsKeyword", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
