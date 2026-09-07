use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_modern_classification_class_properties() {
    let content = r#"class A { 
  private y: number;
  constructor(public x : number, _y : number) { this.y = _y; }
  get z() : number { return this.x + this.y; }
  set a(v: number) { }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
