use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_computed_property_literals() {
    let content = r#"interface I {
    ["foo"](o: any): boolean;
    ["x"]: boolean;
    [1](): string;
    [2]: boolean;
}

class C implements I {}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceComputedPropertyLiterals", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
