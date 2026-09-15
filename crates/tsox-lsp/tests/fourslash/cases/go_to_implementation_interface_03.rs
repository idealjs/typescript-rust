use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_interface_03() {
    let content = r#"interface Fo/*interface_definition*/o { hello: () => void }

var x = <Foo> [|{ hello: () => {} }|];"#;
    let _s = Session::new_for_test("goToImplementationInterface_03", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "interface_definition")
}
