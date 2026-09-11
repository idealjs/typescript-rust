use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_dynamic_import1() {
    let content = r#"// @Filename: foo.ts
/*Destination*/export function foo() { return "foo"; }
import([|"./f/*1*/oo"|])
var x = import([|"./fo/*2*/o"|])"#;
    let mut s = Session::new_for_test("goToDefinitionDynamicImport1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1", "2")
}
