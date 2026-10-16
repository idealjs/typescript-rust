use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_in_function_call() {
    let content = r#"var items = [];
items.forEach(item => {
    for (/**/
});"#;
    let _s = Session::new_for_test("signatureHelpInFunctionCall", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "")
}
