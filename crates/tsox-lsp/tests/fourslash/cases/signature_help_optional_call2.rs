use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_optional_call2() {
    let content = r#"// @strict: false
declare const fnTest: undefined | ((str: string, num: number) => void);
fnTest?.(/*1*/);"#;
    let mut s = Session::new_for_test("signatureHelpOptionalCall2", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "fnTest(str: string, num: number
}
