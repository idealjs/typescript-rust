use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_with_unknown_vs() {
    let content = r#"eval(\/*1*/"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
