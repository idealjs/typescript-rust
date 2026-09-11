use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_index_signatures_both() {
    let content = r#"interface I {
    [x: number]: I;
    [y: string]: I;
}

class C implements I {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceIndexSignaturesBoth", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
