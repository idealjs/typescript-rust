use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_construct_expression() {
    let content = r#"class sampleCls { constructor(str: string, num: number) { } }
var x = new sampleCls(/*1*/"", /*2*/5);"#;
    let mut s = Session::new_for_test("signatureHelpConstructExpression", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "sampleCls(str: string, num: num
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{ParameterName: "num", ParameterSpan: "
}
