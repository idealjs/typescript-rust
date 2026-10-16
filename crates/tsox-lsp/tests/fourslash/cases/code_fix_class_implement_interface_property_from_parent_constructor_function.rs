use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_class_implement_interface_property_from_parent_constructor_function() {
    let content = r#"class A {
    constructor(public x: number) { }
}

class B implements A {[| |]}"#;
    let _s = Session::new_for_test("codeFixClassImplementInterfacePropertyFromParentConstructorFunction", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
