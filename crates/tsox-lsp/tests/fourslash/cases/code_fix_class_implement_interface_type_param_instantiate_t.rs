use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_type_param_instantiate_t() {
    let content = r#"interface I<T> { x: T; }
class C<T> implements I<T> {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceTypeParamInstantiateT", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
