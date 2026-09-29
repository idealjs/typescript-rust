use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_apparent_type_properties() {
    let content = r#"interface Number {
    /*definition*/myObjectMethod(): number;
}

var o = 0;
o.[|/*reference1*/myObjectMethod|]();
o[[|"/*reference2*/myObjectMethod"|]]();"#;
    let _s = Session::new_for_test("goToDefinitionApparentTypeProperties", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "reference1", "reference2")
}
