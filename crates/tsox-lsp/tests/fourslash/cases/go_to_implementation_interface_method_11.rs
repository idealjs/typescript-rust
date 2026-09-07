use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_method_11() {
    let content = r#"interface Foo {
   hel/*reference*/lo(): void;
}

var x = <Foo> { [|hello|]: () => {} };
var y = <Foo> (((({ [|hello|]: () => {} }))));"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "reference")
}
