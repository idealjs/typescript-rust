use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_completions_imports_map1() {
    let content = r##"// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
    "module": "nodenext",
    "lib": ["es5"],
    "rootDir": "src",
    "outDir": "dist"
  }
}
// @Filename: /home/src/workspaces/project/package.json
{
  "type": "module",
  "imports": {
    "#is-browser": {
      "browser": "./dist/env/browser.js",
      "default": "./dist/env/node.js"
    }
  }
}
// @Filename: /home/src/workspaces/project/src/env/browser.ts
export const isBrowser = true;
// @Filename: /home/src/workspaces/project/src/env/node.ts
export const isBrowser = false;
// @Filename: /home/src/workspaces/project/src/a.ts
import {} from "/*1*/";"##;
    let mut s = Session::new_for_test("importCompletions_importsMap1", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
