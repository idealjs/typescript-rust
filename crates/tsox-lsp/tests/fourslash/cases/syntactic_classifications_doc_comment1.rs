use tsox_lsp::fourslash::{self, Session};


#[test]
fn syntactic_classifications_doc_comment1() {
    let content = r#"/** @type {number} */
var v;"#;
    let mut s = Session::new_for_test("syntacticClassificationsDocComment1", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
