use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_relative_path_to_monorepo_package() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
    "module": "nodenext",
    "lib": ["es5"]
  }
}
// @Filename: /home/src/workspaces/project/packages/app/dist/index.d.ts
import {} from "utils";
export const app: number;
// @Filename: /home/src/workspaces/project/packages/utils/package.json
{ "name": "utils", "version": "1.0.0", "main": "dist/index.js" }
// @Filename: /home/src/workspaces/project/packages/utils/dist/index.d.ts
export const x: number;
// @link: /home/src/workspaces/project/packages/utils -> /home/src/workspaces/project/packages/app/node_modules/utils
// @Filename: /home/src/workspaces/project/script.ts
import {} from "./packages/app/dist/index.js";
x/**/"#;
    let mut s = Session::new_for_test("autoImportRelativePathToMonorepoPackage", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"./packages/utils/dist/index.js"}, nil /*preferenc
}
