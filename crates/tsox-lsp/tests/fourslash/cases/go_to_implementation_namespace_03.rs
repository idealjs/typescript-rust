use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_namespace_03() {
    let content = r#"namespace Foo {
    export interface Bar {
        hello(): void;
    }

    class [|BarImpl|] implements Bar {
        hello() {}
    }
}

class [|Baz|] implements Foo.Bar {
    hello() {}
}

var someVar1 : Foo.Bar = [|{ hello: () => {/**1*/} }|];

var someVar2 = <Foo.Bar> [|{ hello: () => {/**2*/} }|];

function whatever(x: Foo.Ba/*reference*/r) {

}"#;
    let _s = Session::new_for_test("goToImplementationNamespace_03", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference")
}
