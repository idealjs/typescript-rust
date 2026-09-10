use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_index_signatures_number() {
    let content = r#"interface I {
    [x: number]: I;
}
class C implements I {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceIndexSignaturesNumber", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
