use tsox_lsp::fourslash::Session;


#[test]
fn definition_name_on_enum_member() {
    let content = r#"enum e {
    firstMember,
    secondMember,
    thirdMember
}
var enumMember = e.[|/*1*/thirdMember|];"#;
    let _s = Session::new_for_test("definitionNameOnEnumMember", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "1")
}
