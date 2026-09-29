use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_dynamic_import3() {
    let content = r#"// @Filename: foo.ts
export function /*Destination*/bar() { return "bar"; }
import('./foo').then(({ [|ba/*1*/r|] }) => undefined);"#;
    let _s = Session::new_for_test("goToDefinitionDynamicImport3", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
