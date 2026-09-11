use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_rest_args3() {
    let content = r#"// @target: esnext
// @lib: esnext
const layers = Object.assign({}, /*1*/...[]);"#;
    let mut s = Session::new_for_test("signatureHelpRestArgs3", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
