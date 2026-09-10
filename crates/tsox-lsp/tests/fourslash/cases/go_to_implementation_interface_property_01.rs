use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_property_01() {
    let content = r#"interface Foo { hello: number }

class Bar implements Foo {
    [|hello|] = 5 * 9;
}

function whatever(foo: Foo) {
    foo.he/*reference*/llo;
}"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceProperty_01", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "reference")
}
