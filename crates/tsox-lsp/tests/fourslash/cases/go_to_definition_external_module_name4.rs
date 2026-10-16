use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_external_module_name4() {
    let content = r#"// @Filename: b.ts
import n = require('unknown/*1*/');"#;
    let _s = Session::new_for_test("goToDefinitionExternalModuleName4", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
