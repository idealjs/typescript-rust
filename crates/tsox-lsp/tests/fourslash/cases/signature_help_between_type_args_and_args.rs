use tsox_lsp::fourslash::Session;


#[test]
fn signature_help_token_crash2() {
    let content = r#"
function foo<T, U>(x: string, y: T, z: U) {

}

foo<number,number>/*1*/("hello", 123,456)
"#;
    let _s = Session::new_for_test("signatureHelpTokenCrash2", content);
    // TODO: f.VerifySignatureHelpWithCases(t, &fourslash.SignatureHelpCase{
}
