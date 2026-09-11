use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_private_constructor_call() {
    let content = r#"class A {
    private constructor() {}
}
var x = new A(/*1*/"#;
    let mut s = Session::new_for_test("quickInfoOnPrivateConstructorCall", content);
    // TODO: f.VerifyNoSignatureHelpForMarkers(t, "1")
}
