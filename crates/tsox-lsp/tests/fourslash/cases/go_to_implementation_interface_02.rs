use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_02() {
    let content = r#"interface Fo/*interface_definition*/o { hello: () => void }

let x: number = 9;

function createFoo(): Foo {
    if (x === 2) {
        return [|{
            hello() {}
        }|];
    }
    return [|{
        hello() {}
    }|];
}

let createFoo2 = (): Foo => [|({hello() {}})|];

function createFooLike() {
    return {
        hello() {}
    };
}"#;
    let mut s = Session::new_for_test("goToImplementationInterface_02", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "interface_definition")
}
