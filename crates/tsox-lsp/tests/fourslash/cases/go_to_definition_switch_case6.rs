use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_switch_case6() {
    let content = r#"export default { [|/*a*/case|] };
[|/*b*/default|];
[|/*c*/case|] 42;"#;
    let mut s = Session::new_for_test("goToDefinitionSwitchCase6", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "a", "b", "c")
}
