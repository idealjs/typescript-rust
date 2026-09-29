use tsox_lsp::fourslash::Session;


#[test]
fn syntactic_classifications_doc_comment4() {
    let content = r#"/** @param {number} p1 */
function foo(p1) {}"#;
    let _s = Session::new_for_test("syntacticClassificationsDocComment4", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
