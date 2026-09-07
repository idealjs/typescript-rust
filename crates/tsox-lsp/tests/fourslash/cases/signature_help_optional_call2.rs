use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_optional_call2() {
    let content = r#"// @strict: false
declare const fnTest: undefined | ((str: string, num: number) => void);
fnTest?.(/*1*/);"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "fnTest(str: string, num: number
}
