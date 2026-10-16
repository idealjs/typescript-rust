use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_on_private_constructor_call() {
    let content = r#"class A {
    private constructor() {}
}
var x = new A(/*1*/"#;
    let _s = Session::new_for_test("quickInfoOnPrivateConstructorCall", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "1")
}
