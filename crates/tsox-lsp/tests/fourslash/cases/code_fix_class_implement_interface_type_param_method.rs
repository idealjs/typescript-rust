use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_type_param_method() {
    let content = r#"interface I {
    f<T extends number>(x: T): T;
}
class C implements I {}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceTypeParamMethod", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
