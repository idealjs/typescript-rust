use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_with_ambient_signatures3() {
    let content = r#"declare abstract class A {
    abstract method(): void;
}
class B implements A {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceWithAmbientSignatures3", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
