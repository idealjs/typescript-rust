use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_class_property_type_query() {
    let content = r#"// @strict: false
class A {
    A: typeof A;
}
class D implements A {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementClassPropertyTypeQuery", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
