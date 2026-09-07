use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_classification1() {
    let content = r#"module /*0*/M {
    export interface /*1*/I {
    }
}
interface /*2*/X extends /*3*/M./*4*/I { }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
