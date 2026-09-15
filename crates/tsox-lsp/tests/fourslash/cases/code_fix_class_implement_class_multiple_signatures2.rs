use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_class_multiple_signatures2() {
    let content = r#"class A {
    method(a: any, b: string): boolean;
    method(a: string, b: number): Function;
    method(a: string): Function;
    method(a: string | number, b?: string | number): boolean | Function { return a + b as any; }
}
class C implements A { }"#;
    let _s = Session::new_for_test("codeFixClassImplementClassMultipleSignatures2", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
