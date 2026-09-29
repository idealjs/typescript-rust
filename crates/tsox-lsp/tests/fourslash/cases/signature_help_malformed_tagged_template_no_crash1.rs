use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_malformed_tagged_template_no_crash1() {
    let content = r#"`${1}
/*m1*/
// ``
"#;
    let mut s = Session::new_for_test("signatureHelpMalformedTaggedTemplateNoCrash1", content);
    fourslash::go_to_marker(&mut s, "m1");
    // TODO: f.VerifyNoSignatureHelpWithContext(t, &lsproto.SignatureHelpContext{
}
