use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_this_01() {
    let content = r#"class [|Bar|] extends Foo {
    hello(): th/*this_type*/is {
        return this;
    }
}"#;
    let _s = Session::new_for_test("goToImplementationThis_01", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "this_type")
}
