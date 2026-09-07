use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn syntactic_classifications_jsx1() {
    let content = r#"// @Filename: file1.tsx
let x  = <div a = "some-value" b = {1}>
    some jsx text
</div>;

let y = <element attr="123"/>"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
