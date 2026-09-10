use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToTypeDefinition"]
#[test]
fn go_to_type_definition_enum_members() {
    let content = r#"enum E {
    value1,
    /*definition*/value2
}
var x = E.value2;

/*reference*/x;"#;
    let mut s = Session::new_for_test("goToTypeDefinitionEnumMembers", content);
    fourslash::unsupported("VerifyBaselineGoToTypeDefinition"); // f.VerifyBaselineGoToTypeDefinition(t, "reference")
}
