use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    let mut s = Session::new_for_test("getOccurrencesReadonly3", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "")
}
