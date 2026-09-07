use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_inherits_abstract_method() {
    let content = r#"abstract class C1 { }
abstract class C2 {
    abstract fＡ<T extends number>(): T;
}
interface I1 extends C1, C2 { }
class C3 implements I1 {[| |]}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
