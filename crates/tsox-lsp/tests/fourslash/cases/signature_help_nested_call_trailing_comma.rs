use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // Both outer and inner calls must have trailing commas, and"]
#[test]
fn signature_help_nested_call_trailing_comma() {
    // TODO: // Regression test for crash when requesting signature help on a call target
    // TODO: // where the nested call has a trailing comma.
    // TODO: // Both outer and inner calls must have trailing commas, and outer must be generic.
    let content = r#"declare function outer<T>(range: T): T;
declare function inner(a: any): any;

outer(inner/*1*/(undefined,),);"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelpPresent"); // f.VerifySignatureHelpPresent(t, &lsproto.SignatureHelpContext{
}
