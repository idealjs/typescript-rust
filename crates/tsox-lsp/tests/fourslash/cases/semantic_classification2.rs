use tsox_lsp::fourslash::Session;


#[test]
fn semantic_classification2() {
    let content = r#"interface /*0*/Thing {
    toExponential(): number;
}

var Thing = 0;
Thing.toExponential();"#;
    let _s = Session::new_for_test("semanticClassification2", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
