use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_apparent_type_properties() {
    let content = r#"interface Number {
    /*definition*/myObjectMethod(): number;
}

var o = 0;
o.[|/*reference1*/myObjectMethod|]();
o[[|"/*reference2*/myObjectMethod"|]]();"#;
    let mut s = Session::new_for_test("goToDefinitionApparentTypeProperties", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "reference1", "reference2")
}
