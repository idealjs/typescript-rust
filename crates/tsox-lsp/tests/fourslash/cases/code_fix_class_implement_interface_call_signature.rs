use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_call_signature() {
    let content = r#"interface I {
    (x: number, b: string): number;
}
class C implements I {[| |]}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceCallSignature", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
