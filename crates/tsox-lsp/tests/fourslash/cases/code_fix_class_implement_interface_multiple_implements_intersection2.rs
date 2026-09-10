use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_class_implement_interface_multiple_implements_intersection2() {
    let content = r#"// @strict: false
interface I1 {
    x: number;
}
interface I2 {
    x: string;
}

class C implements I1,I2 {
    x: string;
}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMultipleImplementsIntersection2", content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
