use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_yield4() {
    let content = r#"function* gen() {
    class C { [/*start*/yield 10]() {} }
}"#;
    let _s = Session::new_for_test("goToDefinitionYield4", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
