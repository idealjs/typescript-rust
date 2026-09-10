use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_class_implement_interface_call_signature() {
    let content = r#"interface I {
    (x: number, b: string): number;
}
class C implements I {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceCallSignature", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
