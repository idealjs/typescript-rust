use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_switch_case6() {
    let content = r#"export default { [|/*a*/case|] };
[|/*b*/default|];
[|/*c*/case|] 42;"#;
    let _s = Session::new_for_test("goToDefinitionSwitchCase6", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "a", "b", "c")
}
