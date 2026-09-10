use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixAvailable"]
#[test]
fn code_fix_class_implement_interface_multiple_implements_intersection1() {
    let content = r#"interface I1 {
    x: number;
}
interface I2 {
    x: string;
}

class C implements I1,I2 {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceMultipleImplementsIntersection1", content);
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, []string{"Implement interface 'I1'", "Implement interface 'I2'"})
}
