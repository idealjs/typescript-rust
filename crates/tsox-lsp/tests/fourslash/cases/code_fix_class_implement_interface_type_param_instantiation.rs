use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_type_param_instantiation() {
    let content = r#"interface I<T> {
   x: T;
}

class C implements I { }"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceTypeParamInstantiation", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
