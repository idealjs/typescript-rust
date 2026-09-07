use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_08() {
    let content = r#"interface Base {
    hello (): void;
}

interface A extends Base {}
interface B extends C, A {}
interface C extends B, A {}

class X implements B {
    [|hello|]() {}
}

function someFunction(d : A) {
    d.he/*function_call*/llo();
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "function_call")
}
