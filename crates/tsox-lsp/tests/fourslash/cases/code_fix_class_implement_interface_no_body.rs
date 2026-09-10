use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixAvailable"]
#[test]
fn code_fix_class_implement_interface_no_body() {
    let content = r#"interface I {
   m(): void
}
class C/*c*/ implements I"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceNoBody", content);
    fourslash::unsupported("VerifyErrorExistsBeforeMarker"); // f.VerifyErrorExistsBeforeMarker(t, "c")
    fourslash::go_to_marker(&mut s, "c");
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, []string{"Implement interface 'I'"})
}
