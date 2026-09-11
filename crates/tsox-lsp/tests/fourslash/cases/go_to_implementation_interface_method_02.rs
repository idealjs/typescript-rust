use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_interface_method_02() {
    let content = r#"interface Foo {
    he/*declaration*/llo(): void
}

abstract class AbstractBar implements Foo {
    abstract hello(): void;
}

class Bar extends AbstractBar {
    [|hello|]() {}
}

function whatever(a: AbstractBar) {
    a.he/*function_call*/llo();
}"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceMethod_02", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "function_call", "declaration")
}
