use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_malformed_ambient_module_export_equals() {
    let content = r#"// @Filename: /a.d.ts
declare moduleu "m" {
  interface A { x: 1 }
  function f(): A[];
  /*m*/export = f;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "m")
}
