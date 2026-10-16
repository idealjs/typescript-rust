use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_skipped_args1() {
    let content = r#"function fn(a: number, b: number, c: number) {}
fn(/*1*/, /*2*/, /*3*/, /*4*/, /*5*/);"#;
    let _s = Session::new_for_test("signatureHelpSkippedArgs1", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
