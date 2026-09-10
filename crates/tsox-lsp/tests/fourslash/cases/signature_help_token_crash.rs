use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelpWithCases"]
#[test]
fn signature_help_token_crash() {
    let content = r#"
function foo(a: any, b: any) {

}

foo((/*1*/

/** This is a JSDoc comment */
foo/** More comments*/((/*2*/
"#;
    let mut s = Session::new_for_test("signatureHelpTokenCrash", content);
    fourslash::unsupported("VerifySignatureHelpWithCases"); // f.VerifySignatureHelpWithCases(t, &fourslash.SignatureHelpCase{
}
