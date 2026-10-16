use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_external_module_name9() {
    let content = r#"// @Filename: b.ts
export * from [|'e/*1*/'|];
// @Filename: a.ts
declare module /*2*/"e" {
    class Foo { }
}"#;
    let _s = Session::new_for_test("goToDefinitionExternalModuleName9", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
