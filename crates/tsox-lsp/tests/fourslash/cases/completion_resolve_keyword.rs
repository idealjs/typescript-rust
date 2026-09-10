use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_resolve_keyword() {
    let content = r#"class C {
	/*a*/
}"#;
    let mut s = Session::new_for_test("completionResolveKeyword", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
