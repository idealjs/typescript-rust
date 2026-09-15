use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_super_00() {
    let content = r#"class [|Foo|] {
    constructor() {}
}

class Bar extends Foo {
    constructor() {
        su/*super_call*/per();
    }
}"#;
    let _s = Session::new_for_test("goToImplementationSuper_00", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "super_call")
}
