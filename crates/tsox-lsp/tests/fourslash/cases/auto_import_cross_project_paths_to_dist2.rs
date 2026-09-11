use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_cross_project_paths_to_dist2() {
    let content = r#"// @Filename: /home/src/workspaces/project/common/tsconfig.json
{
  "compilerOptions": {
    "lib": ["es5"],
    "module": "commonjs",
    "outDir": "dist",
    "composite": true
  },
  "include": ["src"]
}
// @Filename: /home/src/workspaces/project/common/src/MyModule.ts
export function square(n: number) {
  return n * 2;
}
// @Filename: /home/src/workspaces/project/web/tsconfig.json
{
  "compilerOptions": {
    "lib": ["es5"],
    "module": "esnext",
    "moduleResolution": "node",
    "noEmit": true,
    "paths": {
      "@common/*": ["../common/dist/src/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "../common" }]
}
// @Filename: /home/src/workspaces/project/web/src/MyApp.ts
import { square } from "@common/MyModule";
// @Filename: /home/src/workspaces/project/web/src/Helper.ts
export function saveMe() {
  square/**/(2);
}"#;
    let mut s = Session::new_for_test("autoImportCrossProject_paths_toDist2", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_file(&mut s, "/home/src/workspaces/project/web/src/Helper.ts");
    // TODO: f.VerifyImportFixModuleSpecifiers(t, "", []string{"@common/MyModule"}, &lsutil.UserPreferences{Impor
}
