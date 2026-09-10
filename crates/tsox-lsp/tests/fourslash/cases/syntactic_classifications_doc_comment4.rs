use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn syntactic_classifications_doc_comment4() {
    let content = r#"/** @param {number} p1 */
function foo(p1) {}"#;
    let mut s = Session::new_for_test("syntacticClassificationsDocComment4", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
