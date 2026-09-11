use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_resolve_keyword() {
    let content = r#"class C {
	/*a*/
}"#;
    let mut s = Session::new_for_test("completionResolveKeyword", content);
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
