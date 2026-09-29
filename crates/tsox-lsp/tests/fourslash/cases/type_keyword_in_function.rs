use tsox_lsp::fourslash::{self, Session};


#[test]
fn type_keyword_in_function() {
    let content = r#"function a() {
    ty/**/
}"#;
    let mut s = Session::new_for_test("typeKeywordInFunction", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
