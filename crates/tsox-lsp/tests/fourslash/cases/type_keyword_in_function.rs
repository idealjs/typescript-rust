use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn type_keyword_in_function() {
    let content = r#"function a() {
    ty/**/
}"#;
    let mut s = Session::new_for_test("typeKeywordInFunction", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
