use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_empty_multiline_body() {
    let content = r#"// @lib: es2017
interface I {
    x: number;
    y: number;
}
class C implements I {
}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceEmptyMultilineBody", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
