use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_optional_call() {
    let content = r#"function fnTest(str: string, num: number) { }
fnTest?.(/*1*/);"#;
    let mut s = Session::new_for_test("signatureHelpOptionalCall", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "fnTest(str: string, num: number
}
