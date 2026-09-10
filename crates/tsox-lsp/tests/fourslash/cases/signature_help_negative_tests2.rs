use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpForMarkers"]
#[test]
fn signature_help_negative_tests2() {
    let content = r#"class clsOverload { constructor(); constructor(test: string); constructor(test?: string) { } }
var x = new clsOverload/*beforeOpenParen*/()/*afterCloseParen*/;"#;
    let mut s = Session::new_for_test("signatureHelpNegativeTests2", content);
    fourslash::unsupported("VerifyNoSignatureHelpForMarkers"); // f.VerifyNoSignatureHelpForMarkers(t, "beforeOpenParen", "afterCloseParen")
}
