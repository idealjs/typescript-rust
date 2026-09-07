use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_cross_project_paths_strip_src() {
    let content = r#"// @Filename: /home/src/workspaces/project/packages/app/package.json
{ "name": "app", "dependencies": { "dep": "*" } }
// @Filename: /home/src/workspaces/project/packages/app/tsconfig.json
{
  "compilerOptions": {
    "lib": ["es5"],
    "module": "commonjs",
    "outDir": "dist",
    "rootDir": "src",
    "baseUrl": ".",
    "paths": {
      "dep": ["../dep/src/main"],
      "dep/*": ["../dep/src/*"]
    }
  }
  "references": [{ "path": "../dep" }]
}
// @Filename: /home/src/workspaces/project/packages/app/src/index.ts
dep1/*1*/;
// @Filename: /home/src/workspaces/project/packages/app/src/utils.ts
dep2/*2*/;
// @Filename: /home/src/workspaces/project/packages/app/src/a.ts
import "dep";
// @Filename: /home/src/workspaces/project/packages/dep/package.json
{ "name": "dep", "main": "dist/main.js", "types": "dist/main.d.ts" }
// @Filename: /home/src/workspaces/project/packages/dep/tsconfig.json
{
  "compilerOptions": { "lib": ["es5"], "outDir": "dist", "rootDir": "src", "module": "commonjs" }
}
// @Filename: /home/src/workspaces/project/packages/dep/src/main.ts
import "./sub/folder";
export const dep1 = 0;
// @Filename: /home/src/workspaces/project/packages/dep/src/sub/folder/index.ts
export const dep2 = 0;"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
