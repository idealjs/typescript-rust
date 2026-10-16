use tsox_lsp::fourslash::Session;


#[test]
fn export_in_object_literal() {
    let content = r#"// @Filename: a.ts
const k = {
    [|export|] f() { }
}"#;
    let _s = Session::new_for_test("exportInObjectLiteral", content);
    // TODO: f.VerifyBaselineDocumentHighlightsWithOptions(t, nil /*preferences*/, []string{f.Ranges()[0].FileNam
}
