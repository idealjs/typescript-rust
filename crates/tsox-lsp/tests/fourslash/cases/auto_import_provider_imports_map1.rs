use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_provider_imports_map1() {
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
isBrowser/**/"##;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"#is-browser", "./env/browser.js"}, nil /*preferen
}
