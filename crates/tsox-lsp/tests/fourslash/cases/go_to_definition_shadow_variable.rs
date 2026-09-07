use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_shadow_variable() {
    let content = r#"var shadowVariable = "foo";
function shadowVariableTestModule() {
    var /*shadowVariableDefinition*/shadowVariable;
    /*shadowVariableReference*/shadowVariable = 1;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "shadowVariableReference")
}
