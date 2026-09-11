use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_interface_method_03() {
    let content = r#"interface Foo {
    hello (): void;
}

class Bar extends SuperBar {
    [|hello|]() {}
}

class SuperBar implements Foo {
    hello() {} // should not show up
}

class OtherBar implements Foo {
    hello() {} // should not show up
}

new Bar().hel/*function_call*/lo();
new Bar()["hello"]();"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceMethod_03", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "function_call")
}
