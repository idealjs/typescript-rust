use tsox_lsp::fourslash::{self, Session};


#[test]
fn syntactic_classifications_for_of_keyword() {
    let content = r#"for (var of of of) { }"#;
    let mut s = Session::new_for_test("syntacticClassificationsForOfKeyword", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
