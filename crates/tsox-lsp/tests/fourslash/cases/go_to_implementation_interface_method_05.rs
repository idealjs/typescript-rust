use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_method_05() {
    let content = r#"interface Foo {
    hello (): void;
}

class SuperBar implements Foo {
    [|hello|]() {}
}

class Bar extends SuperBar {
    hello2() {}
}

class OtherBar extends SuperBar {
    hello() {}
    hello2() {}
    hello3() {}
}

class NotRelatedToBar {
    hello() {}         // Equivalent to last case, but shares no common ancestors with Bar and so is not returned
    hello2() {}
    hello3() {}
}

class NotBar extends SuperBar {
    hello() {}         // Should not be returned because it is not structurally equivalent to Bar
}

function whatever(x: Bar) {
    x.he/*function_call*/llo()
}"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceMethod_05", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "function_call")
}
