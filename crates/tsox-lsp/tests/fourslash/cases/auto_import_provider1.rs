use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_provider1() {
    let content = r#"// @Filename: /home/src/workspaces/project/node_modules/@angular/forms/package.json
{ "name": "@angular/forms", "typings": "./forms.d.ts" }
// @Filename: /home/src/workspaces/project/node_modules/@angular/forms/forms.d.ts
export class PatternValidator {}
// @Filename: /home/src/workspaces/project/tsconfig.json
{ "compilerOptions": { "lib": ["es5"] } }
// @Filename: /home/src/workspaces/project/package.json
{ "dependencies": { "@angular/forms": "*" } }
// @Filename: /home/src/workspaces/project/index.ts
PatternValidator/**/"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: opts654 := f.GetOptions()
    // TODO: opts654.FormatCodeSettings.NewLineCharacter = "\n"
    fourslash::unsupported("Configure"); // f.Configure(t, opts654)
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
