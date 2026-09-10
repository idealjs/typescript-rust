use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_modern_classification_interfaces() {
    let content = r#"interface Pos { x: number, y: number };
const p = { x: 1, y: 2 } as Pos;
const foo = (o: Pos) => o.x + o.y;"#;
    let mut s = Session::new_for_test("semanticModernClassificationInterfaces", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
