use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_skipped_args1_vs() {
    let content = r#"function fn(a: number, b: number, c: number) {}
fn(/*1*/, /*2*/, /*3*/, /*4*/, /*5*/);"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
