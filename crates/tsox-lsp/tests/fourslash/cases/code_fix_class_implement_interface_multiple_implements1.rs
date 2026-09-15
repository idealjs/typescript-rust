use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_multiple_implements1() {
    let content = r#"// @strict: false
interface I1 {
    x: number;
}
interface I2 {
    y: number;
}

class C implements I1,I2 {[|
    |]y: number;
}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceMultipleImplements1", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
