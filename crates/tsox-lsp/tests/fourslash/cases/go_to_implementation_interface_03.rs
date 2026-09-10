use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_03() {
    let content = r#"interface Fo/*interface_definition*/o { hello: () => void }

var x = <Foo> [|{ hello: () => {} }|];"#;
    let mut s = Session::new_for_test("goToImplementationInterface_03", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "interface_definition")
}
