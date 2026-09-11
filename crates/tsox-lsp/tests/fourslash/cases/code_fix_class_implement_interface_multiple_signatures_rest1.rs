use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_multiple_signatures_rest1() {
    let content = r#"interface I {
    method(a: number, ...b: string[]): boolean;
    method(a: string, ...b: number[]): Function;
    method(a: string): Function;
}

class C implements I {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMultipleSignaturesRest1", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
