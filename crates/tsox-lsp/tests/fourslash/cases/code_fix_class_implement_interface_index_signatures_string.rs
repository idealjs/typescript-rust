use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_index_signatures_string() {
    let content = r#"interface I<X> {
    [Ƚ: string]: X;
}

class C implements I<number> {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceIndexSignaturesString", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
