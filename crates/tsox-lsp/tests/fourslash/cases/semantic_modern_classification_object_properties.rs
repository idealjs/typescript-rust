use tsox_lsp::fourslash::Session;


#[test]
fn semantic_modern_classification_object_properties() {
    let content = r#"let x = 1, y = 1;
const a1 = { e: 1 };
var a2 = { x };"#;
    let _s = Session::new_for_test("semanticModernClassificationObjectProperties", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
