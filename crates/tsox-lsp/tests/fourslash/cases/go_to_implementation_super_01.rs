use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_super_01() {
    let content = r#"class [|Foo|] {
    hello() {}
}

class Bar extends Foo {
    hello() {
        sup/*super_call*/er.hello();
    }
}"#;
    let _s = Session::new_for_test("goToImplementationSuper_01", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "super_call")
}
