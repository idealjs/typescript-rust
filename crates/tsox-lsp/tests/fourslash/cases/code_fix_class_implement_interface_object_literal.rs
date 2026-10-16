use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_object_literal() {
    let content = r#"interface IPerson {
    coordinate: {
        x: number;
        y: number;
    }
}
class Person implements IPerson { }"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceObjectLiteral", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
