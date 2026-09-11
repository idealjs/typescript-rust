use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_class_implement_interface_property_from_parent_constructor_function() {
    let content = r#"class A {
    constructor(public x: number) { }
}

class B implements A {[| |]}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfacePropertyFromParentConstructorFunction", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
