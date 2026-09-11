use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_object_literal() {
    let content = r#"var objectLiteral = { n: 5, s: "", f: (a: number, b: string) => "" };
objectLiteral.f(/*objectLiteral1*/4, /*objectLiteral2*/"");"#;
    let mut s = Session::new_for_test("signatureHelpObjectLiteral", content);
    fourslash::go_to_marker(&mut s, "objectLiteral1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(a: number, b: string): string
    fourslash::go_to_marker(&mut s, "objectLiteral2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "f(a: number, b: string): string
}
