use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_simple_function_call() {
    let content = r#"// Simple function test
function functionCall(str: string, num: number) {
}
functionCall(/*functionCall1*/);
functionCall("", /*functionCall2*/1);"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "functionCall1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "functionCall(str: string, num: 
    fourslash::go_to_marker(&mut s, "functionCall2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "functionCall(str: string, num: 
}
