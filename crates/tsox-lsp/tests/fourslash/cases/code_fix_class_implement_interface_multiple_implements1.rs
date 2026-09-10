use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
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
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMultipleImplements1", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
