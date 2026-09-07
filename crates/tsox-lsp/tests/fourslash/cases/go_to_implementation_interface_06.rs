use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_06() {
    let content = r#"interface Fo/*interface_definition*/o {
    new (a: number): SomeOtherType;
}

interface SomeOtherType {}

let x: Foo = [|class { constructor (a: number) {} }|];
let y = <Foo> [|class { constructor (a: number) {} }|];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "interface_definition")
}
