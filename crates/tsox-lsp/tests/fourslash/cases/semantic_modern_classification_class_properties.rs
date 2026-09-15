use tsox_lsp::fourslash::Session;


#[test]
fn semantic_modern_classification_class_properties() {
    let content = r#"class A { 
  private y: number;
  constructor(public x : number, _y : number) { this.y = _y; }
  get z() : number { return this.x + this.y; }
  set a(v: number) { }
}"#;
    let _s = Session::new_for_test("semanticModernClassificationClassProperties", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
