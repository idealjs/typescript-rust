use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_shorthand_property_assignment_00() {
    let content = r#"interface Foo {
    someFunction(): void;
}

interface FooConstructor {
    new (): Foo
}

interface Bar {
    Foo: FooConstructor;
}

var x = class /*classExpression*/Foo {
    createBarInClassExpression(): Bar {
        return {
            Fo/*classExpressionRef*/o
        };
    }

    someFunction() {}
}

class /*declaredClass*/Foo {

}

function createBarUsingClassDeclaration(): Bar {
    return {
        Fo/*declaredClassRef*/o
    };
}"#;
    let _s = Session::new_for_test("goToImplementationShorthandPropertyAssignment_00", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "classExpressionRef", "declaredClassRef")
}
