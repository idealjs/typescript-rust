use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_typeof_this() {
    let content = r#"
// @Filename: /a.ts
interface Foo {
  bar(): typeof [|this|];
}
"#;
    let _s = Session::new_for_test("documentHighlightTypeofThis", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
