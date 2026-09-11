use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_with_interface_as_identifier() {
    let content = r#"interface C {
    (): void;
}
C(/*1*/);"#;
    let mut s = Session::new_for_test("signatureHelpWithInterfaceAsIdentifier", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "1")
}
