use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_classification_uninstantiated_module_with_variable_of_same_name1() {
    let content = r#"declare module /*0*/M {
    interface /*1*/I {

    }
}

var M = { I: 10 };"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
