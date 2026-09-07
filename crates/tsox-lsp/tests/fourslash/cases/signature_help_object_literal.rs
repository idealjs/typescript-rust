use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_object_literal() {
    let content = r#"var objectLiteral = { n: 5, s: "", f: (a: number, b: string) => "" };
objectLiteral.f(/*objectLiteral1*/4, /*objectLiteral2*/"");"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "objectLiteral1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(a: number, b: string): string
    fourslash::go_to_marker(&mut s, "objectLiteral2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(a: number, b: string): string
}
