use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn syntactic_classifications_doc_comment2() {
    let content = r#"/** @param foo { function(x): string } */
var v;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
