use tsox_lsp::fourslash::{self, Session};


#[test]
fn syntactic_classifications_doc_comment2() {
    let content = r#"/** @param foo { function(x): string } */
var v;"#;
    let mut s = Session::new_for_test("syntacticClassificationsDocComment2", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
