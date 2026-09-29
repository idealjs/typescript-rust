use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_undeclared_symbol() {
    let content = r#"interface I {
   x: T;
}

class C implements I { }"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceUndeclaredSymbol", content);
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Implement interface 'I'"})
}
