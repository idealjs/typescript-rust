use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlight_malformed_ambient_module_export_equals() {
    let content = r#"// @Filename: /a.d.ts
declare moduleu "m" {
  interface A { x: 1 }
  function f(): A[];
  /*m*/export = f;
}"#;
    let mut s = Session::new_for_test("documentHighlightMalformedAmbientModuleExportEquals", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "m")
}
