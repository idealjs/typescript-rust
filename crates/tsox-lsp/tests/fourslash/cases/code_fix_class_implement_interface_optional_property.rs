use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_optional_property() {
    let content = r#"// @strict: false
interface IPerson {
    name: string;
    birthday?: string;
}
class Person implements IPerson {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceOptionalProperty", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
