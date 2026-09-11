use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_script_import_server() {
    let content = r#"// @lib: es5
// @Filename: /home/src/workspaces/project/scriptThing.ts
/*1d*/console.log("woooo side effects")
// @Filename: /home/src/workspaces/project/stylez.css
/*2d*/div {
  color: magenta;
}
// @Filename: /home/src/workspaces/project/moduleThing.ts
import [|/*1*/"./scriptThing"|];
import [|/*2*/"./stylez.css"|];
import [|/*3*/"./foo.txt"|];"#;
    let mut s = Session::new_for_test("goToDefinitionScriptImportServer", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1", "2", "3")
}
