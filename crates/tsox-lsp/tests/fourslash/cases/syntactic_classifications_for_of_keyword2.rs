use tsox_lsp::fourslash::Session;


#[test]
fn syntactic_classifications_for_of_keyword2() {
    let content = r#"for (var of in of) { }"#;
    let _s = Session::new_for_test("syntacticClassificationsForOfKeyword2", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
