use tsox_lsp::fourslash::Session;


#[test]
fn syntactic_classifications_doc_comment3() {
    let content = r#"/** @param foo { number /* } */
var v;"#;
    let _s = Session::new_for_test("syntacticClassificationsDocComment3", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
