use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_class_function_void_inferred() {
    let content = r#"class A {
    f() {}
}

class B implements A {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementClassFunctionVoidInferred", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
