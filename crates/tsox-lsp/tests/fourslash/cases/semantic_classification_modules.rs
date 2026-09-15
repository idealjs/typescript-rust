use tsox_lsp::fourslash::Session;


#[test]
fn semantic_classification_modules() {
    let content = r#"module /*0*/M {
    export var v;
    export interface /*1*/I {
    }
}

var x: /*2*/M./*3*/I = /*4*/M.v;
var y = /*5*/M;"#;
    let _s = Session::new_for_test("semanticClassificationModules", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
