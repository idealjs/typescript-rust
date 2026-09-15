use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_on_protected_constructor_call() {
    let content = r#"class A {
    protected constructor() {}
}
var x = new A(/*1*/"#;
    let _s = Session::new_for_test("quickInfoOnProtectedConstructorCall", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "1")
}
