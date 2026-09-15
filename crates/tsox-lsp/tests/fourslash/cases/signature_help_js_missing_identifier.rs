use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_js_missing_identifier() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: test.js
log(/**/)"#;
    let _s = Session::new_for_test("signatureHelpJSMissingIdentifier", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "")
}
