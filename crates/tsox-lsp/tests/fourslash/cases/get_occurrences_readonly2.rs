use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_readonly2() {
    let content = r#"type T = {
  [|readonly|] prop: string;
}"#;
    let _s = Session::new_for_test("getOccurrencesReadonly2", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
