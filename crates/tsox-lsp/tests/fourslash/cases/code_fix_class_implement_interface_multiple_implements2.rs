use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_multiple_implements2() {
    let content = r#"// @strict: false
interface I1 {
    x: number;
}
interface I2 {
    y: "𣋝ઢȴ¬⏊";
}

class C implements I1,I2 {[|
    |]x: number;
}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMultipleImplements2", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
