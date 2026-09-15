use tsox_lsp::fourslash::Session;


#[test]
fn document_highlight_type_parameter_constraint_expression_no_crash1() {
    let content = r#"// @Filename: /a.ts
const v/*m*/alue = 1;
type Box<T extends +value> = typeof value"#;
    let _s = Session::new_for_test("documentHighlightTypeParameterConstraintExpressionNoCrash1", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "m")
}
