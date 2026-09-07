use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn signature_help_js_missing_property_access() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: test.js
foo.filter(/**/)"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
