use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_class_expression() {
    let content = r#"interface I { x: number; }
new class implements I {};"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceClassExpression", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
