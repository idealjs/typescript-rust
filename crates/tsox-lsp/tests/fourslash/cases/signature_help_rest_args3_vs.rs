use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_rest_args3_vs() {
    let content = r#"// @target: esnext
// @lib: esnext
const layers = Object.assign({}, /*1*/...[]);"#;
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
