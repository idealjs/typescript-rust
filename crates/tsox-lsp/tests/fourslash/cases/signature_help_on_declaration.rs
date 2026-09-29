use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_on_declaration() {
    let content = r#"function f</**/
x"#;
    let _s = Session::new_for_test("signatureHelpOnDeclaration", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "")
}
