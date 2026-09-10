use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_export_from() {
    let content = r#"export * /*1*/;
export {} /*2*/;"#;
    let mut s = Session::new_for_test("completionExportFrom", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
