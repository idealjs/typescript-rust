use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn signature_help_rest_args3_vs() {
    let content = r#"// @target: esnext
// @lib: esnext
const layers = Object.assign({}, /*1*/...[]);"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
