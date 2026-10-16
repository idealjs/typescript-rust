use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_js_missing_property_access() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: test.js
foo.filter(/**/)"#;
    let _s = Session::new_for_test("signatureHelpJSMissingPropertyAccess", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
