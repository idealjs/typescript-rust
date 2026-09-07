use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpForMarkers"]
#[test]
fn signature_help_object_creation_expression_no_args_not_available() {
    let content = r#"class sampleCls { constructor(str: string, num: number) { } }
var x = new sampleCls/**/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoSignatureHelpForMarkers"); // f.VerifyNoSignatureHelpForMarkers(t, "")
}
