use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn signature_help_with_unknown() {
    let content = r#"eval(\/*1*/"#;
    let mut s = Session::new_for_test("signatureHelpWithUnknown", content);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
