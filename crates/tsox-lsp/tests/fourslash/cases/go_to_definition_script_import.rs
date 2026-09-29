use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("goToDefinitionScriptImport", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1", "2")
}
