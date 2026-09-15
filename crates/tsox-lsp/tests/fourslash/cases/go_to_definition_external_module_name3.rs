use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_external_module_name3() {
    let content = r#"// @Filename: b.ts
import n = require([|'e/*1*/'|]);
var x = new n.Foo();
// @Filename: a.ts
declare module /*2*/"e" {
    class Foo { }
}"#;
    let _s = Session::new_for_test("goToDefinitionExternalModuleName3", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
