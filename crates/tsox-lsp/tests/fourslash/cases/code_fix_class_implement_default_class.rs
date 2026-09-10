use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_default_class() {
    let content = r#"interface I { x: number; }
export default class implements I {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementDefaultClass", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
