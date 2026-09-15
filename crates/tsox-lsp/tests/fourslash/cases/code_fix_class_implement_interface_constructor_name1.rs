use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_constructor_name1() {
    let content = r#"interface I {
    constructor: number;
}
class C implements I {}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceConstructorName1", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
