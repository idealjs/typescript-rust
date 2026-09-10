use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_expando_class2() {
    let content = r#"// @strict: true
// @allowJs: true
// @checkJs: true
// @filename: index.js
const Core = {}

Core.Test = class {
  constructor() { }
}

Core.Test.prototype.foo = 10

new Core.Tes/*1*/t()"#;
    let mut s = Session::new_for_test("goToDefinitionExpandoClass2", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
