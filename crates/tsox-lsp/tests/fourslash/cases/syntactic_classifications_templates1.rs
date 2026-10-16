use tsox_lsp::fourslash::Session;


#[test]
fn syntactic_classifications_templates1() {
    let content = r#"var v = 10e0;
var x = {
    p1: `hello world`,
    p2: `goodbye ${0} cruel ${0} world`,
};"#;
    let _s = Session::new_for_test("syntacticClassificationsTemplates1", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
