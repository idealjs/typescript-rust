use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_cross_project_symlinks_to_src() {
    let content = r#"// @Filename: /home/src/workspaces/project/packages/app/package.json
{ "name": "app", "dependencies": { "dep": "*" } }
// @Filename: /home/src/workspaces/project/packages/app/tsconfig.json
{
  "compilerOptions": {
    "lib": ["es5"],
    "module": "commonjs",
    "outDir": "dist",
    "rootDir": "src",
    "baseUrl": "."
  }
  "references": [{ "path": "../dep" }]
}
// @Filename: /home/src/workspaces/project/packages/app/src/index.ts
dep/**/
// @Filename: /home/src/workspaces/project/packages/dep/package.json
{ "name": "dep", "main": "dist/index.js", "types": "dist/index.d.ts" }
// @Filename: /home/src/workspaces/project/packages/dep/tsconfig.json
{
  "compilerOptions": { "lib": ["es5"], "outDir": "dist", "rootDir": "src", "module": "commonjs" }
}
// @Filename: /home/src/workspaces/project/packages/dep/src/index.ts
import "./sub/folder";
// @Filename: /home/src/workspaces/project/packages/dep/src/sub/folder/index.ts
export const dep = 0;
// @link: /home/src/workspaces/project/packages/dep -> /home/src/workspaces/project/packages/app/node_modules/dep"#;
    let mut s = Session::new_for_test("autoImportCrossProject_symlinks_toSrc", content);
    // TODO: f.Configure(t, lsutil.UserPreferences{AutoImportEntrypointDirectorySearch: core.TSTrue})
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
