use tsox_lsp::fourslash::Session;


#[test]
fn go_to_type_definition_enum_members() {
    let content = r#"enum E {
    value1,
    /*definition*/value2
}
var x = E.value2;

/*reference*/x;"#;
    let _s = Session::new_for_test("goToTypeDefinitionEnumMembers", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference")
}
