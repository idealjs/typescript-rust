use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_type_param_instantiate_deeply() {
    let content = r#"interface I<T> {
    x: { y: T, z: T[] };
}
class C implements I<number> {[| |]}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceTypeParamInstantiateDeeply", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
