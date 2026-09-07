use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_shadow_variable_inside_module() {
    let content = r#"namespace shdModule {
    var /*shadowVariableDefinition*/shdVar;
    /*shadowVariableReference*/shdVar = 1;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "shadowVariableReference")
}
