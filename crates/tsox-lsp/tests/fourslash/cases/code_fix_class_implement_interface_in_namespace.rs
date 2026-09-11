use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_in_namespace() {
    let content = r#"namespace N1 {
    export interface I1 {
        f1():string;
    }
}
interface I1 {
    f1();
}

class C1 implements N1.I1 {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceInNamespace", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
