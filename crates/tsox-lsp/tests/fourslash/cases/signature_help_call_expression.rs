use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_call_expression() {
    let content = r#"function fnTest(str: string, num: number) { }
fnTest(/*1*/'', /*2*/5);"#;
    let mut s = Session::new_for_test("signatureHelpCallExpression", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "fnTest(str: string, num: number
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "num", ParameterSpan: "
}
