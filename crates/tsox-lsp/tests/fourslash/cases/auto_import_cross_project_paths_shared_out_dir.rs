use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_cross_project_paths_shared_out_dir() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.base.json
{
  "compilerOptions": {
    "lib": ["es5"],
    "module": "commonjs",
    "baseUrl": ".",
    "paths": {
      "packages/*": ["./packages/*"]
    }
  }
}
// @Filename: /home/src/workspaces/project/packages/app/tsconfig.json
{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": { "outDir": "../../dist/packages/app" },
  "references": [{ "path": "../dep" }]
}
// @Filename: /home/src/workspaces/project/packages/app/index.ts
dep/**/
// @Filename: /home/src/workspaces/project/packages/app/utils.ts
import "packages/dep";
// @Filename: /home/src/workspaces/project/packages/dep/tsconfig.json
{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": { "outDir": "../../dist/packages/dep" }
}
// @Filename: /home/src/workspaces/project/packages/dep/index.ts
import "./sub/folder";
// @Filename: /home/src/workspaces/project/packages/dep/sub/folder/index.ts
export const dep = 0;"#;
    let mut s = Session::new_for_test("autoImportCrossProject_paths_sharedOutDir", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
