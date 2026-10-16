use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_yield1() {
    let content = r#"function* /*end1*/gen() {
    [|/*start1*/yield|] 0;
}

const /*end2*/genFunction = function*() {
    [|/*start2*/yield|] 0;
}"#;
    let _s = Session::new_for_test("goToDefinitionYield1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "start1", "start2")
}
