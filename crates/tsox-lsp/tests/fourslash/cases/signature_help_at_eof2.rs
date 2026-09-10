use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpForMarkersWithContext"]
#[test]
fn signature_help_at_eof2() {
    let content = r#"console.log()
/**/"#;
    let mut s = Session::new_for_test("signatureHelpAtEOF2", content);
    fourslash::unsupported("VerifyNoSignatureHelpForMarkersWithContext"); // f.VerifyNoSignatureHelpForMarkersWithContext(t, &lsproto.SignatureHelpContext{TriggerKind: lsproto.S
}
