use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_index_type() {
    let content = r#"interface I<X> {
    x: keyof X;
}
class C<Y> implements I<Y> {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceIndexType", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
