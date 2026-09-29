use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_enum_01() {
    let content = r#"enum [|Foo|] {
    Foo1 = function initializer() { return 5 } (),
    Foo2 = 6
}

Fo/*reference*/o;"#;
    let _s = Session::new_for_test("goToImplementationEnum_01", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference")
}
