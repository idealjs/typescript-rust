use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn syntactic_classifications_conflict_markers2() {
    let content = r#"<<<<<<< HEAD
class C { }
=======
class D { }
>>>>>>> Branch - a"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
