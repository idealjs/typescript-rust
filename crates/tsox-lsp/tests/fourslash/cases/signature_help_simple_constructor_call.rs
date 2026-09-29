use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_simple_constructor_call() {
    let content = r#"class ConstructorCall {
    constructor(str: string, num: number) {
    }
}
var x = new ConstructorCall(/*constructorCall1*/1,/*constructorCall2*/2);"#;
    let mut s = Session::new_for_test("signatureHelpSimpleConstructorCall", content);
    fourslash::go_to_marker(&mut s, "constructorCall1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "ConstructorCall(str: string, nu
    fourslash::go_to_marker(&mut s, "constructorCall2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "ConstructorCall(str: string, nu
}
