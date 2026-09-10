use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_before_semicolon1() {
    let content = r#"function Foo(arg1: string, arg2: string) {
}

Foo(/**/;"#;
    let mut s = Session::new_for_test("signatureHelpBeforeSemicolon1", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "Foo(arg1: string, arg2: string)
}
