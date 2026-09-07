use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn semantic_modern_classification_members() {
    let content = r#"class A {
  static x = 9;
  f = 9;
  async m() { return A.x + await this.m(); };
  get s() { return this.f; 
  static t() { return new A().f; };
  constructor() {}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
    // TODO: }
}
