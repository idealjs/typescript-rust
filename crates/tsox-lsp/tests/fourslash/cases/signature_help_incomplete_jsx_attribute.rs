use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_incomplete_jsx_attribute() {
    let content = r#"// @Filename: /a.tsx
<a><b c=
/*a*/</a>"#;
    let mut s = Session::new_for_test("signatureHelpIncompleteJsxAttribute", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "a")
}
