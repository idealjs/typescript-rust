use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_at_eof() {
    let content = r#"function Foo(arg1: string, arg2: string) {
}

Foo(/**/"#;
    let mut s = Session::new_for_test("signatureHelpAtEOF", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "Foo(arg1: string, arg2: string)
}
