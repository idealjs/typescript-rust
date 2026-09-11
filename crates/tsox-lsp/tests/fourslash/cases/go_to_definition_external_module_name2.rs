use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_external_module_name2() {
    let content = r#"// @Filename: b.ts
import n = require([|'./a/*1*/'|]);
var x = new n.Foo();
// @Filename: a.ts
/*2*/class Foo {}
export var x = 0;"#;
    let mut s = Session::new_for_test("goToDefinitionExternalModuleName2", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
