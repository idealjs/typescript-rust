use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_expando_class1() {
    let content = r#"// @strict: true
// @allowJs: true
// @checkJs: true
// @filename: index.js
const Core = {}

Core.Test = class { }

Core.Test.prototype.foo = 10

new Core.Tes/*1*/t()"#;
    let _s = Session::new_for_test("goToDefinitionExpandoClass1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
