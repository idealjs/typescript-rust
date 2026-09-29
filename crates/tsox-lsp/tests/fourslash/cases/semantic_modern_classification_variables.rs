use tsox_lsp::fourslash::Session;


#[test]
fn semantic_modern_classification_variables() {
    let content = r#"  var x = 9, y1 = [x];
  try {
    for (const s of y1) { x = s }
  } catch (e) {
    throw y1;
  }"#;
    let _s = Session::new_for_test("semanticModernClassificationVariables", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
