use tsox_lsp::fourslash::Session;


#[test]
fn semantic_classification1() {
    let content = r#"module /*0*/M {
    export interface /*1*/I {
    }
}
interface /*2*/X extends /*3*/M./*4*/I { }"#;
    let _s = Session::new_for_test("semanticClassification1", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
