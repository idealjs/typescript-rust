use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_method_this_and_self_reference() {
    let content = r#"interface I {
    f(x: number, y: this): I
}

class C implements I {[| |]}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceMethodThisAndSelfReference", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
