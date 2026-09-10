use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn signature_help_anonymous_type() {
    let content = r#"const comparers: Array<(a: any, b: any) => boolean> = [];

comparers.push((a,/**/ b) => true);"#;
    let mut s = Session::new_for_test("signatureHelpAnonymousType", content);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
