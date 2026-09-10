use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_on_invalid_parameter_decorator() {
    let content = r#"function f(@/*1*/f) {}"#;
    let mut s = Session::new_for_test("goToDefinitionOnInvalidParameterDecorator", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
