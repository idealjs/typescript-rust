use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_class_multiple_signatures2() {
    let content = r#"class A {
    method(a: any, b: string): boolean;
    method(a: string, b: number): Function;
    method(a: string): Function;
    method(a: string | number, b?: string | number): boolean | Function { return a + b as any; }
}
class C implements A { }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
