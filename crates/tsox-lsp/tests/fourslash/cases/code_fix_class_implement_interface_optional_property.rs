use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_optional_property() {
    let content = r#"// @strict: false
interface IPerson {
    name: string;
    birthday?: string;
}
class Person implements IPerson {}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceOptionalProperty", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
