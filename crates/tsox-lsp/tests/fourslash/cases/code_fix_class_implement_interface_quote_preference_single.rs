use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_quote_preference_single() {
    let content = r#"interface I {
    a(): void;
    b(x: 'x', y: 'a' | 'b'): 'b';

    c: 'c';
    d: { e: 'e'; };
}
class Foo implements I {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
