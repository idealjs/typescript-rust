use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_with_unknown() {
    let content = r#"eval(\/*1*/"#;
    let _s = Session::new_for_test("signatureHelpWithUnknown", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
