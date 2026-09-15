use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_expando_element_access() {
    let content = r#"function f() {}
f[/*0*/"x"] = 0;
f[[|/*1*/"x"|]] = 1;"#;
    let _s = Session::new_for_test("goToDefinitionExpandoElementAccess", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
