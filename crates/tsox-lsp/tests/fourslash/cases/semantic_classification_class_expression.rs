use tsox_lsp::fourslash::Session;


#[test]
fn semantic_classification_class_expression() {
    let content = r#"var x = class /*0*/C {}
class /*1*/C {}
class /*2*/D extends class /*3*/B{} { }"#;
    let _s = Session::new_for_test("semanticClassificationClassExpression", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
