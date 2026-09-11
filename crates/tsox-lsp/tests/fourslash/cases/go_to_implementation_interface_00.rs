use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_interface_00() {
    let content = r#"interface Fo/*interface_definition*/o {
    hello: () => void
}

interface Baz extends Foo {}

var bar: Foo = [|{|"parts": ["(","object literal",")"], "kind": "interface"|}{ hello: helloImpl /**0*/ }|];
var baz: Foo[] = [|[{ hello: helloImpl /**4*/ }]|];

function helloImpl () {}

function whatever(x: Foo = [|{|"parts": ["(","object literal",")"], "kind": "interface"|}{ hello() {/**1*/} }|] ) {
}

class Bar {
    x: Foo = [|{ hello() {/*2*/} }|]

    constructor(public f: Foo = [|{ hello() {/**3*/} }|] ) {}
}"#;
    let mut s = Session::new_for_test("goToImplementationInterface_00", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "interface_definition")
}
