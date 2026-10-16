use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_inherits_abstract_method() {
    let content = r#"abstract class C1 { }
abstract class C2 {
    abstract fＡ<T extends number>(): T;
}
interface I1 extends C1, C2 { }
class C3 implements I1 {[| |]}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceInheritsAbstractMethod", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
