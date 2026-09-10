use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySignatureHelpWithCases"]
#[test]
fn signature_help_token_crash2() {
    let content = r#"
function foo<T, U>(x: string, y: T, z: U) {

}

foo<number,number>/*1*/("hello", 123,456)
"#;
    let mut s = Session::new_for_test("signatureHelpTokenCrash2", content);
    fourslash::unsupported("VerifySignatureHelpWithCases"); // f.VerifySignatureHelpWithCases(t, &fourslash.SignatureHelpCase{
}
