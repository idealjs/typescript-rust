use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_interface_method_08() {
    let content = r#"interface Foo {
    hello (): void;
}

class SuperBar implements Foo {
   [|hello|]() {}
}

class Bar extends SuperBar {
   whatever() { this.he/*function_call*/llo(); }
}

class SubBar extends Bar {
   [|hello|]() {}
}"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceMethod_08", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "function_call")
}
