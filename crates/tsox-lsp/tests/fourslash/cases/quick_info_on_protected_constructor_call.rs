use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpForMarkers"]
#[test]
fn quick_info_on_protected_constructor_call() {
    let content = r#"class A {
    protected constructor() {}
}
var x = new A(/*1*/"#;
    let mut s = Session::new_for_test("quickInfoOnProtectedConstructorCall", content);
    fourslash::unsupported("VerifyNoSignatureHelpForMarkers"); // f.VerifyNoSignatureHelpForMarkers(t, "1")
}
