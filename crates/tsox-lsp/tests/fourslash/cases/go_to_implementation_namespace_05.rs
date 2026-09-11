use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_namespace_05() {
    let content = r#"namespace /*implementation0*/Foo./*implementation2*/Baz {
    export function hello() {}
}

module /*implementation1*/Bar./*implementation3*/Baz {
    export function sure() {}
}

let x = Fo/*reference0*/o;
let y = Ba/*reference1*/r;
let x1 = Foo.B/*reference2*/az;
let y1 = Bar.B/*reference3*/az;"#;
    let mut s = Session::new_for_test("goToImplementationNamespace_05", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference0", "reference1", "reference2", "reference3")
}
