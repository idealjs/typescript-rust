use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn document_highlight_type_parameter_constraint_expression_no_crash1() {
    let content = r#"// @Filename: /a.ts
const v/*m*/alue = 1;
type Box<T extends +value> = typeof value"#;
    let mut s = Session::new_for_test("documentHighlightTypeParameterConstraintExpressionNoCrash1", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "m")
}
