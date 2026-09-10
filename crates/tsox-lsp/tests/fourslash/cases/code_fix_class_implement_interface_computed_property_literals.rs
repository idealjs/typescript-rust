use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_computed_property_literals() {
    let content = r#"interface I {
    ["foo"](o: any): boolean;
    ["x"]: boolean;
    [1](): string;
    [2]: boolean;
}

class C implements I {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceComputedPropertyLiterals", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
