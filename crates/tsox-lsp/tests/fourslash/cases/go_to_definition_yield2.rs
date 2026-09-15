use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_yield2() {
    let content = r#"function* outerGen() {
    function* /*end*/gen() {
        [|/*start*/yield|] 0;
    }
    return gen
}"#;
    let _s = Session::new_for_test("goToDefinitionYield2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start")
}
