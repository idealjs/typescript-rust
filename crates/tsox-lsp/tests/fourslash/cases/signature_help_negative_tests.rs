use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpForMarkers"]
#[test]
fn signature_help_negative_tests() {
    let content = r#"//inside a comment foo(/*insideComment*/
cl/*invalidContext*/ass InvalidSignatureHelpLocation { }
InvalidSignatureHelpLocation(/*validContext*/);"#;
    let mut s = Session::new_for_test("signatureHelpNegativeTests", content);
    fourslash::unsupported("VerifyNoSignatureHelpForMarkers"); // f.VerifyNoSignatureHelpForMarkers(t, "insideComment", "invalidContext", "validContext")
}
