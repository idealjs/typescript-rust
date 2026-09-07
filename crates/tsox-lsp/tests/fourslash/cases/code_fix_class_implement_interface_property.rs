use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_property() {
    let content = r#"// @lib: es2017
enum E { a,b,c }
interface I {
    x: E;
    y: E.a
    z: symbol;
    w: object;
}
class C implements I {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
