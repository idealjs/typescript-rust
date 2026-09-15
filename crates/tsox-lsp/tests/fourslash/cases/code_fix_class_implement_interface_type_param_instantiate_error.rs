use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_type_param_instantiate_error() {
    let content = r#"interface I<T extends string> {
   x: T;
}

class C implements I<number> { }"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfaceTypeParamInstantiateError", content);
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Implement interface 'I<number>'"})
}
