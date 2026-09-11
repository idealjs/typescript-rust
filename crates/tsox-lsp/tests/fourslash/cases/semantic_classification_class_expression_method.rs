use tsox_lsp::fourslash::{self, Session};


#[test]
fn semantic_classification_class_expression_method() {
    let content = r#"var x = class C {
  equals(other: C) { return this == other; }
};"#;
    let mut s = Session::new_for_test("semanticClassificationClassExpressionMethod", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
