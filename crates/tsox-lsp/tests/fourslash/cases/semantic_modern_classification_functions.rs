use tsox_lsp::fourslash::Session;


#[test]
fn semantic_modern_classification_functions() {
    let content = r#"function foo(p1) {
  return foo(Math.abs(p1))
}
`/${window.location}`.split("/").forEach(s => foo(s));"#;
    let _s = Session::new_for_test("semanticModernClassificationFunctions", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
