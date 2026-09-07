use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_method_04() {
    let content = r#"interface Foo {
    hello (): void;
}

class Bar extends SuperBar {
    [|hello|]() {}
}

class SuperBar implements Foo {
    [|hello|]() {}
}

class OtherBar implements Foo {
    hello() {} // should not show up
}

function (x: SuperBar) {
    x.he/*function_call*/llo()
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "function_call")
}
