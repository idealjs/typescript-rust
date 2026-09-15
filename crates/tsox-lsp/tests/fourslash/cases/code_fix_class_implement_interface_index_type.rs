use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_index_type() {
    let content = r#"interface I<X> {
    x: keyof X;
}
class C<Y> implements I<Y> {[| |]}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceIndexType", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
