use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_function_parameter() {
    let content = r#"function parameterFunction(callback: (a: number, b: string) => void) {
    callback(/*parameterFunction1*/5, /*parameterFunction2*/"");
}"#;
    let mut s = Session::new_for_test("signatureHelpFunctionParameter", content);
    fourslash::go_to_marker(&mut s, "parameterFunction1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "callback(a: number, b: string):
    fourslash::go_to_marker(&mut s, "parameterFunction2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "callback(a: number, b: string):
}
