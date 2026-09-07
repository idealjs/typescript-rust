use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_multiple_signatures_rest2() {
    let content = r#"interface I {
    method(a: number, ...b: string[]): boolean;
    method(a: string, b: number): Function;
    method(a: string): Function;
}

class C implements I {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
