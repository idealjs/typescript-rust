use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_mapped_type1() {
    let content = r#"interface I<X> {
    x: { readonly [K in keyof X]: X[K] };
}
class C<Y> implements I<Y> {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMappedType1", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
