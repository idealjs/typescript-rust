use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_array_tuple() {
    let content = r#"interface I {
    x: number[];
    y: Array<number>;
    z: [number, string, I];
}

class C implements I {[| |]}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceArrayTuple", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
