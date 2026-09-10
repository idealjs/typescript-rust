use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_dynamic_import3() {
    let content = r#"// @Filename: foo.ts
export function /*Destination*/bar() { return "bar"; }
import('./foo').then(({ [|ba/*1*/r|] }) => undefined);"#;
    let mut s = Session::new_for_test("goToDefinitionDynamicImport3", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
