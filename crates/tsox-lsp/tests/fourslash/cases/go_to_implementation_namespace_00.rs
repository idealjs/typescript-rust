use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_namespace_00() {
    let content = r#"namespace /*implementation0*/Foo {
    export function hello() {}
}

module /*implementation1*/Bar {
    export function sure() {}
}

let x = Fo/*reference0*/o;
let y = Ba/*reference1*/r;"#;
    let mut s = Session::new_for_test("goToImplementationNamespace_00", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference0", "reference1")
}
