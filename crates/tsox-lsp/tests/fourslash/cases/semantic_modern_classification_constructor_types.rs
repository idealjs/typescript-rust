use tsox_lsp::fourslash::{self, Session};


#[test]
fn semantic_modern_classification_constructor_types() {
    let content = r#"// @lib: es5
Object.create(null);
const x = Promise.resolve(Number.MAX_VALUE);
if (x instanceof Promise) {}"#;
    let mut s = Session::new_for_test("semanticModernClassificationConstructorTypes", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
