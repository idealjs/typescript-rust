use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_with_negative_number() {
    let content = r#"interface X { value: -1 | 0 | 1; }
class Y implements X { }"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceWithNegativeNumber", content);
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Implement interface 'X'"})
}
