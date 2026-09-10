use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_primitives() {
    let content = r#"var x: st/*primitive*/ring;"#;
    let mut s = Session::new_for_test("goToDefinitionPrimitives", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "primitive")
}
