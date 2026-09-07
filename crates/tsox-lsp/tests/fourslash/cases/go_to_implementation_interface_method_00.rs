use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_method_00() {
    let content = r#"interface Foo {
    he/*declaration*/llo: () => void
}

var bar: Foo = { [|hello|]: helloImpl };
var baz: Foo = { "[|hello|]": helloImpl };

function helloImpl () {}

function whatever(x: Foo = { [|hello|]() {/**1*/} }) {
    x.he/*function_call*/llo()
}

class Bar {
    x: Foo = { [|hello|]() {/*2*/} }

    constructor(public f: Foo = { [|hello|]() {/**3*/} } ) {}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "function_call", "declaration")
}
