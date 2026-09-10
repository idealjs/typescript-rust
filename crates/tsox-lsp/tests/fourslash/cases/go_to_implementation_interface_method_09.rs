use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_method_09() {
    let content = r#"interface Foo {
    hello (): void;
}

class SubBar extends Bar {
    hello() {}
}

class Bar extends SuperBar {
    hello() {}

    whatever() {
        super.he/*function_call*/llo();
        super["hel/*element_access*/lo"]();
    }
}

class SuperBar extends MegaBar {
    [|hello|]() {}
}

class MegaBar implements Foo {
    hello() {}
}"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceMethod_09", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "function_call", "element_access")
}
