use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_token_crash() {
    let content = r#"
function foo(a: any, b: any) {

}

foo((/*1*/

/** This is a JSDoc comment */
foo/** More comments*/((/*2*/
"#;
    let _s = Session::new_for_test("signatureHelpTokenCrash", content);
    // TODO: f.VerifySignatureHelpWithCases(t, &fourslash.SignatureHelpCase{
    // TODO: f.VerifySignatureHelpWithCases(t, &fourslash.SignatureHelpCase{
}
