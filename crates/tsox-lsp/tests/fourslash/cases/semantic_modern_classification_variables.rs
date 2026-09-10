use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_modern_classification_variables() {
    let content = r#"  var x = 9, y1 = [x];
  try {
    for (const s of y1) { x = s }
  } catch (e) {
    throw y1;
  }"#;
    let mut s = Session::new_for_test("semanticModernClassificationVariables", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
