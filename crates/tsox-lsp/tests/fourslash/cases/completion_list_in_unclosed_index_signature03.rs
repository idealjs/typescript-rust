use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn completion_list_in_unclosed_index_signature03() {
    let content = r#"class C {
    [foo: string]: { x: typeof /*1*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: }
}
