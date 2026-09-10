use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_modern_classification_functions() {
    let content = r#"function foo(p1) {
  return foo(Math.abs(p1))
}
` + "`" + `/${window.location}` + "`" + `.split("/").forEach(s => foo(s));"#;
    let mut s = Session::new_for_test("semanticModernClassificationFunctions", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
