use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_method_01() {
    let content = r#"interface Foo {
    hel/*declaration*/lo(): void;
    okay?: number;
}

class Bar implements Foo {
    [|hello|]() {}
    public sure() {}
}

function whatever(a: Foo) {
    a.he/*function_call*/llo();
}

whatever(new Bar());"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceMethod_01", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "function_call", "declaration")
}
