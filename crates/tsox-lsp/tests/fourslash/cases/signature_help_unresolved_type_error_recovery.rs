use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_unresolved_type_in_error_recovered_signature() {
    let content = r#"function f(x: {
    a?: U =
    b?: (p: U) => void
}) {}
f(/*a*/);"#;
    let mut s = Session::new_for_test("signatureHelpUnresolvedTypeInErrorRecoveredSignature", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
