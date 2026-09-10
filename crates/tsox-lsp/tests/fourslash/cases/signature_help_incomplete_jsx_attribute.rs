use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpForMarkers"]
#[test]
fn signature_help_incomplete_jsx_attribute() {
    let content = r#"// @Filename: /a.tsx
<a><b c=
/*a*/</a>"#;
    let mut s = Session::new_for_test("signatureHelpIncompleteJsxAttribute", content);
    fourslash::unsupported("VerifyNoSignatureHelpForMarkers"); // f.VerifyNoSignatureHelpForMarkers(t, "a")
}
