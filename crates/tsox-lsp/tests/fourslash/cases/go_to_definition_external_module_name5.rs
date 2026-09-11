use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_external_module_name5() {
    let content = r#"// @Filename: a.ts
declare module /*2*/[|"external/*1*/"|] {
    class Foo { }
}"#;
    let mut s = Session::new_for_test("goToDefinitionExternalModuleName5", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
