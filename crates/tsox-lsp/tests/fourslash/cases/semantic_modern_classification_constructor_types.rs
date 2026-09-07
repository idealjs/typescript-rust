use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_modern_classification_constructor_types() {
    let content = r#"// @lib: es5
Object.create(null);
const x = Promise.resolve(Number.MAX_VALUE);
if (x instanceof Promise) {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
