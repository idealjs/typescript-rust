use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_at_dotted_namespace() {
    let content = r#"namespace wwer./**/w"#;
    let mut s = Session::new_for_test("completionAtDottedNamespace", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
