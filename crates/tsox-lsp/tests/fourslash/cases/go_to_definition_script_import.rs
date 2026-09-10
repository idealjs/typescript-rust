use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_script_import() {
    let content = r#"// @filename: scriptThing.ts
/*1d*/console.log("woooo side effects")
// @filename: stylez.css
/*2d*/div {
  color: magenta;
}
// @filename: moduleThing.ts
import [|/*1*/"./scriptThing"|];
import [|/*2*/"./stylez.css"|];"#;
    let mut s = Session::new_for_test("goToDefinitionScriptImport", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1", "2")
}
