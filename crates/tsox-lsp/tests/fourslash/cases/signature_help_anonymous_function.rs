use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_anonymous_function() {
    let content = r#"var anonymousFunctionTest = function(n: number, s: string): (a: number, b: string) => string {
    return null;
}
anonymousFunctionTest(5, "")(/*anonymousFunction1*/1, /*anonymousFunction2*/"");"#;
    let mut s = Session::new_for_test("signatureHelpAnonymousFunction", content);
    fourslash::go_to_marker(&mut s, "anonymousFunction1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "(a: number, b: string): string"
    fourslash::go_to_marker(&mut s, "anonymousFunction2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "b", ParameterSpan: "b:
}
