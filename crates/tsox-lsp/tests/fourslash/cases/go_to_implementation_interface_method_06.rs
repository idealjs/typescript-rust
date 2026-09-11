use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_interface_method_06() {
    let content = r#"interface SuperFoo {
    hello (): void;
}

interface Foo extends SuperFoo {
    someOtherFunction(): void;
}

class Bar implements Foo {
     [|hello|]() {}
     someOtherFunction() {}
}

function createFoo(): Foo {
    return {
        [|hello|]() {},
        someOtherFunction() {}
    };
}

var y: Foo = {
    [|hello|]() {},
    someOtherFunction() {}
};

class FooLike implements SuperFoo {
     hello() {}
     someOtherFunction() {}
}

class NotRelatedToFoo {
     hello() {}                // This case is equivalent to the last case, but is not returned because it does not share a common ancestor with Foo
     someOtherFunction() {}
}

class NotFoo implements SuperFoo {
     hello() {}                // We only want implementations of Foo, even though the function is declared in SuperFoo
}

function (x: Foo) {
    x.he/*function_call*/llo()
}"#;
    let mut s = Session::new_for_test("goToImplementationInterfaceMethod_06", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "function_call")
}
