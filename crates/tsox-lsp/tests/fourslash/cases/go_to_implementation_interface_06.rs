use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_interface_06() {
    let content = r#"interface Fo/*interface_definition*/o {
    new (a: number): SomeOtherType;
}

interface SomeOtherType {}

let x: Foo = [|class { constructor (a: number) {} }|];
let y = <Foo> [|class { constructor (a: number) {} }|];"#;
    let _s = Session::new_for_test("goToImplementationInterface_06", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "interface_definition")
}
