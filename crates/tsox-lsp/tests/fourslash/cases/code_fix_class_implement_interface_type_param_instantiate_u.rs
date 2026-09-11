use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_type_param_instantiate_u() {
    let content = r#"interface I<T> { x: T; }
class C<U> implements I<U> {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceTypeParamInstantiateU", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
