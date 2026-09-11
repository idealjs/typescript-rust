use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_member_type_alias() {
    let content = r#"type MyType = [string, number];
interface I { x: MyType; test(a: MyType): void; }
class C implements I {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMemberTypeAlias", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
