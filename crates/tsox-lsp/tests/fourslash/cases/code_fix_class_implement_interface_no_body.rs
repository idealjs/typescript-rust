use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_no_body() {
    let content = r#"interface I {
   m(): void
}
class C/*c*/ implements I"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceNoBody", content);
    // TODO: f.VerifyErrorExistsBeforeMarker(t, "c")
    fourslash::go_to_marker(&mut s, "c");
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Implement interface 'I'"})
}
