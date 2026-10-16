use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_namespace_conflict() {
    let content = r#"namespace N1 {
    export interface I1 { x: number; }
}
interface I1 {
    f1();
}
class C1 implements N1.I1 {}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceNamespaceConflict", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
