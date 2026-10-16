use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_class_multiple_signatures1() {
    let content = r#"class A {
    method(a: number, b: string): boolean;
    method(a: string | number, b?: string | number): boolean | Function { return a + b as any; }
}
class C implements A {}"#;
    let _s = Session::new_for_test("codeFixClassImplementClassMultipleSignatures1", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
