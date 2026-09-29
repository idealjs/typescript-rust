use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_class_function_void_inferred() {
    let content = r#"class A {
    f() {}
}

class B implements A {[| |]}"#;
    let _s = Session::new_for_test("codeFixClassImplementClassFunctionVoidInferred", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
