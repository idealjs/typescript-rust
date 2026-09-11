use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_js_missing_identifier() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: test.js
log(/**/)"#;
    let mut s = Session::new_for_test("signatureHelpJSMissingIdentifier", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "")
}
