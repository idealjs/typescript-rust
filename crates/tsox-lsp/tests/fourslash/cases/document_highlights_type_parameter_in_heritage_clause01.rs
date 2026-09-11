use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_highlights_type_parameter_in_heritage_clause01() {
    let content = r#"// @lib: es5
interface I<[|T|]> extends I<[|T|]>, [|T|] {
}"#;
    let mut s = Session::new_for_test("documentHighlightsTypeParameterInHeritageClause01", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
