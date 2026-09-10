use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_typeof_this() {
    let content = r#"
// @Filename: /a.ts
interface Foo {
  bar(): typeof [|this|];
}
"#;
    let mut s = Session::new_for_test("documentHighlightTypeofThis", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, f.Ranges()[0])
}
