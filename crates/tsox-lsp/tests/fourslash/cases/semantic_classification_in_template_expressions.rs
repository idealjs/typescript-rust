use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn semantic_classification_in_template_expressions() {
    let content = r#"module /*0*/M {
    export class /*1*/C {
        static x;
    }
    export enum /*2*/E {
        E1 = 0
    }
}
` + "`" + `abcd${ /*3*/M./*4*/C.x + /*5*/M./*6*/E.E1}efg` + "`" + `"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
