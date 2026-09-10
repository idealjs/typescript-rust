use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_shorthand_property_assignment_02() {
    let content = r#"interface Foo {
	 hello(): void;
}

function createFoo(): Foo {
    return {
         hello
    };

    function [|hello|]() {}
}

function whatever(x: Foo) {
     x.h/*function_call*/ello();
}"#;
    let mut s = Session::new_for_test("goToImplementationShorthandPropertyAssignment_02", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "function_call")
}
