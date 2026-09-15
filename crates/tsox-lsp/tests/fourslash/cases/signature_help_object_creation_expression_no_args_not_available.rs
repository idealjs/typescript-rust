use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_object_creation_expression_no_args_not_available() {
    let content = r#"class sampleCls { constructor(str: string, num: number) { } }
var x = new sampleCls/**/;"#;
    let _s = Session::new_for_test("signatureHelpObjectCreationExpressionNoArgs_NotAvailable", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "")
}
