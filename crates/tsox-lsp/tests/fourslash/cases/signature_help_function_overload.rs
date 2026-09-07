use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_function_overload() {
    let content = r#"function functionOverload();
function functionOverload(test: string);
function functionOverload(test?: string) { }
functionOverload(/*functionOverload1*/);
functionOverload(""/*functionOverload2*/);"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "functionOverload1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "functionOverload(): any", Param
    fourslash::go_to_marker(&mut s, "functionOverload2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "functionOverload(test: string):
}
