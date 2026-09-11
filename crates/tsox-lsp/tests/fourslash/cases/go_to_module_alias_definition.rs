use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_module_alias_definition() {
    let content = r#"// @Filename: a.ts
export class /*2*/Foo {}
// @Filename: b.ts
 import /*3*/n = require('a');
 var x = new [|/*1*/n|].Foo();"#;
    let mut s = Session::new_for_test("goToModuleAliasDefinition", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
