use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_interface_after_implement() {
    let content = r#"interface /*interfaceDefinition*/sInt {
    sVar: number;
    sFn: () => void;
}

class iClass implements /*interfaceReference*/sInt {
    public sVar = 1;
    public sFn() {
    }
}"#;
    let mut s = Session::new_for_test("goToDefinitionInterfaceAfterImplement", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "interfaceReference")
}
