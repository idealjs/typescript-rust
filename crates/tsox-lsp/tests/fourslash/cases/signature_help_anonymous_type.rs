use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_anonymous_type() {
    let content = r#"const comparers: Array<(a: any, b: any) => boolean> = [];

comparers.push((a,/**/ b) => true);"#;
    let _s = Session::new_for_test("signatureHelpAnonymousType", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
