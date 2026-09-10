use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpForMarkers"]
#[test]
fn signature_help_on_declaration() {
    let content = r#"function f</**/
x"#;
    let mut s = Session::new_for_test("signatureHelpOnDeclaration", content);
    fourslash::unsupported("VerifyNoSignatureHelpForMarkers"); // f.VerifyNoSignatureHelpForMarkers(t, "")
}
