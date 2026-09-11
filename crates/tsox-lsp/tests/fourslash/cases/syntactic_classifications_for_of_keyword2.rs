use tsox_lsp::fourslash::{self, Session};


#[test]
fn syntactic_classifications_for_of_keyword2() {
    let content = r#"for (var of in of) { }"#;
    let mut s = Session::new_for_test("syntacticClassificationsForOfKeyword2", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
