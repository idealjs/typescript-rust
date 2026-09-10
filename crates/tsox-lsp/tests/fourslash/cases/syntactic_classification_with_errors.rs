use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn syntactic_classification_with_errors() {
    let content = r#"class A {
    a:
}
c ="#;
    let mut s = Session::new_for_test("syntacticClassificationWithErrors", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
