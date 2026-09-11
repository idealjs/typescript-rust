use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_interface_method_11() {
    let content = r#"interface Foo {
   hel/*reference*/lo(): void;
}

var x = <Foo> { [|hello|]: () => {} };
var y = <Foo> (((({ [|hello|]: () => {} }))));"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceMethod_11", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference")
}
