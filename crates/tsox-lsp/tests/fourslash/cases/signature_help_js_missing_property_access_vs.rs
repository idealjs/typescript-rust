use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_js_missing_property_access_vs() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: test.js
foo.filter(/**/)"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
