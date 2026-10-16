use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_external_module_name6() {
    let content = r#"// @Filename: b.ts
import * from [|'e/*1*/'|];
// @Filename: a.ts
declare module /*2*/"e" {
    class Foo { }
}"#;
    let _s = Session::new_for_test("goToDefinitionExternalModuleName6", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
