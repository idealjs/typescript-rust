use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_interface_property_00() {
    let content = r#"interface Foo {
    hello: number
}

var bar: Foo = { [|hello|]: 5 };


function whatever(x: Foo = { [|hello|]: 5 * 9 }) {
    x.he/*reference*/llo
}

class Bar {
    x: Foo = { [|hello|]: 6 }

    constructor(public f: Foo = { [|hello|]: 7 } ) {}
}"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceProperty_00", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference")
}
