use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpForMarkers"]
#[test]
fn signature_help_with_interface_as_identifier() {
    let content = r#"interface C {
    (): void;
}
C(/*1*/);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoSignatureHelpForMarkers"); // f.VerifyNoSignatureHelpForMarkers(t, "1")
}
