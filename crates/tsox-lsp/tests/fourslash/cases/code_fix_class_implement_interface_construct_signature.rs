use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_construct_signature() {
    let content = r#"interface I {
    new (x: number, b: string);
}
class C implements I {[| |]}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceConstructSignature", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
