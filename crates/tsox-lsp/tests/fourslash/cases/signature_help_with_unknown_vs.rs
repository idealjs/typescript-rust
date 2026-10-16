use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_with_unknown_vs() {
    let content = r#"eval(\/*1*/"#;
    let _s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
