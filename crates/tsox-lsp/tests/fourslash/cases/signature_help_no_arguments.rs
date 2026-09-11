use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_no_arguments() {
    let content = r#"function foo(n: number): string {
}

foo(/**/"#;
    let mut s = Session::new_for_test("signatureHelpNoArguments", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(n: number): string", Parame
}
