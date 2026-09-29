use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_simple_function_call() {
    let content = r#"// Simple function test
function functionCall(str: string, num: number) {
}
functionCall(/*functionCall1*/);
functionCall("", /*functionCall2*/1);"#;
    let mut s = Session::new_for_test("signatureHelpSimpleFunctionCall", content);
    fourslash::go_to_marker(&mut s, "functionCall1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "functionCall(str: string, num: 
    fourslash::go_to_marker(&mut s, "functionCall2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "functionCall(str: string, num: 
}
