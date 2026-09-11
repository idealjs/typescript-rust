use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_anonymous_type() {
    let content = r#"const comparers: Array<(a: any, b: any) => boolean> = [];

comparers.push((a,/**/ b) => true);"#;
    let mut s = Session::new_for_test("signatureHelpAnonymousType", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
