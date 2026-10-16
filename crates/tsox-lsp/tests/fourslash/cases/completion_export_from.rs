use tsox_lsp::fourslash::Session;


#[test]
fn completion_export_from() {
    let content = r#"export * /*1*/;
export {} /*2*/;"#;
    let _s = Session::new_for_test("completionExportFrom", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
