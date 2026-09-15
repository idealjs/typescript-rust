use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_multiple_signatures_rest2() {
    let content = r#"interface I {
    method(a: number, ...b: string[]): boolean;
    method(a: string, b: number): Function;
    method(a: string): Function;
}

class C implements I {}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceMultipleSignaturesRest2", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
