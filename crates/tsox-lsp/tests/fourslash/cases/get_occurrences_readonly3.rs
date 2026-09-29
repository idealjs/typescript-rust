use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_readonly3() {
    let content = r#"class C {
  [|readonly|] prop: /**/readonly string[] = [];
  constructor([|readonly|] prop2: string) {
    class D {
      readonly prop: string = "";  
    }
  }
}"#;
    let _s = Session::new_for_test("getOccurrencesReadonly3", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
