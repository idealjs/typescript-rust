use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_with_invalid_argument_list1() {
    let content = r#"function foo(a) { }
foo(hello my name /**/is"#;
    let mut s = Session::new_for_test("signatureHelpWithInvalidArgumentList1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "foo(a: any): void"})
}
