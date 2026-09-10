use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_method_10() {
    let content = r#"interface BaseFoo {
	 hello(): void;
}

interface Foo extends BaseFoo {
	 aloha(): void;
}

interface Bar {
 	 hello(): void;
 	 goodbye(): void;
}

class FooImpl implements Foo {
 	 [|hello|]() {/**FooImpl*/}
 	 aloha() {}
}

class BaseFooImpl implements BaseFoo {
 	 hello() {/**BaseFooImpl*/}    // Should not show up
}

class BarImpl implements Bar {
	 [|hello|]() {/**BarImpl*/}
	 goodbye() {}
}

class FooAndBarImpl implements Foo, Bar {
	 [|hello|]() {/**FooAndBarImpl*/}
	 aloha() {}
	 goodbye() {}
}

function someFunction(x: Foo | Bar) {
	 x.he/*function_call0*/llo();
}

function anotherFunction(x: Foo & Bar) {
	 x.he/*function_call1*/llo();
}"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceMethod_10", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "function_call0", "function_call1")
}
