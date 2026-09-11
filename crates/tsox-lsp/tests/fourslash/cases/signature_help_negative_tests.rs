use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_negative_tests() {
    let content = r#"//inside a comment foo(/*insideComment*/
cl/*invalidContext*/ass InvalidSignatureHelpLocation { }
InvalidSignatureHelpLocation(/*validContext*/);"#;
    let mut s = Session::new_for_test("signatureHelpNegativeTests", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "insideComment", "invalidContext", "validContext")
}
