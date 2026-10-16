use tsox_lsp::fourslash::Session;


#[test]
fn occurrences02() {
    let content = r#"// @lib: es5
function [|f|](x: typeof [|f|]) {
    [|f|]([|f|]);
}"#;
    let _s = Session::new_for_test("occurrences02", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
