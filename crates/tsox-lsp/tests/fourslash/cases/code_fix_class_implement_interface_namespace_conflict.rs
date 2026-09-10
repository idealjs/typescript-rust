use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_namespace_conflict() {
    let content = r#"namespace N1 {
    export interface I1 { x: number; }
}
interface I1 {
    f1();
}
class C1 implements N1.I1 {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceNamespaceConflict", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
