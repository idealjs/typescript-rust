use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_in_export1() {
    let content = r#"class [|C|] {}
[|export|] { [|C|] [|as|] [|D|] };"#;
    let _s = Session::new_for_test("documentHighlightInExport1", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
