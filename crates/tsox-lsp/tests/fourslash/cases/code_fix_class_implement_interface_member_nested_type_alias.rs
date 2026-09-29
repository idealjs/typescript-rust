use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_member_nested_type_alias() {
    let content = r#"type Either<T> = { val: T } | Error;
interface I {
    x: Either<Either<string>>;
    foo(x: Either<Either<string>>): void;
}
class C implements I {}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceMemberNestedTypeAlias", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
