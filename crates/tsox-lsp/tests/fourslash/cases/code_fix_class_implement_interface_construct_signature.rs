use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_construct_signature() {
    let content = r#"interface I {
    new (x: number, b: string);
}
class C implements I {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceConstructSignature", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
