use tsox_lsp::fourslash::Session;


#[test]
fn syntactic_classification_with_errors() {
    let content = r#"class A {
    a:
}
c ="#;
    let _s = Session::new_for_test("syntacticClassificationWithErrors", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
