use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("semanticModernClassificationMembers", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
    // TODO: }
}
