use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn syntactic_classifications_templates1() {
    let content = r#"var v = 10e0;
var x = {
    p1: ` + "`" + `hello world` + "`" + `,
    p2: ` + "`" + `goodbye ${0} cruel ${0} world` + "`" + `,
};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
