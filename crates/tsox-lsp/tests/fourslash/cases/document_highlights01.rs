use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlights01() {
    let content = r#"// @lib: es5
// @Filename: a.ts
function [|f|](x: typeof [|f|]) {
    [|f|]([|f|]);
}"#;
    let mut s = Session::new_for_test("documentHighlights01", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
