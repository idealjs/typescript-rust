use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_04() {
    let content = r#"interface Fo/*interface_definition*/o {
    (a: number): void
}

var bar: Foo = [|(a) => {/**0*/}|];

function whatever(x: Foo = [|(a) => {/**1*/}|] ) {
}

class Bar {
    x: Foo = [|(a) => {/**2*/}|]

    constructor(public f: Foo = [|function(a) {}|] ) {}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "interface_definition")
}
