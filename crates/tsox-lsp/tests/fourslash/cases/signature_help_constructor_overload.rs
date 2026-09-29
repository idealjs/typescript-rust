use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_constructor_overload() {
    let content = r#"class clsOverload { constructor(); constructor(test: string); constructor(test?: string) { } }
var x = new clsOverload(/*1*/);
var y = new clsOverload(/*2*/'');"#;
    let mut s = Session::new_for_test("signatureHelpConstructorOverload", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "clsOverload(): clsOverload", Pa
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "clsOverload(test: string): clsO
}
