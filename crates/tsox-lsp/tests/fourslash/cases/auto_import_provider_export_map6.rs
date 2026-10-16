use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_provider_export_map6() {
    let content = r#"// @types package should be ignored because implementation package has types
// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
    "module": "nodenext",
    "lib": ["es5"]
  }
}
// @Filename: /home/src/workspaces/project/package.json
{
  "type": "module",
  "dependencies": {
    "dependency": "^1.0.0"
  },
  "devDependencies": {
    "@types/dependency": "^1.0.0"
  }
}
// @Filename: /home/src/workspaces/project/node_modules/dependency/package.json
{
  "type": "module",
  "name": "dependency",
  "version": "1.0.0",
  "exports": {
    ".": "./lib/index.js",
    "./lol": "./lib/lol.js"
  }
}
// @Filename: /home/src/workspaces/project/node_modules/dependency/lib/index.js
export function fooFromIndex() {}
// @Filename: /home/src/workspaces/project/node_modules/dependency/lib/index.d.ts
export declare function fooFromIndex(): void
// @Filename: /home/src/workspaces/project/node_modules/dependency/lib/lol.js
export function fooFromLol() {}
// @Filename: /home/src/workspaces/project/node_modules/dependency/lib/lol.d.ts
export declare function fooFromLol(): void
// @Filename: /home/src/workspaces/project/node_modules/@types/dependency/package.json
{
  "type": "module",
  "name": "@types/dependency",
  "version": "1.0.0",
  "exports": {
    ".": "./lib/index.d.ts",
    "./lol": "./lib/lol.d.ts"
  }
}
// @Filename: /home/src/workspaces/project/node_modules/@types/dependency/lib/index.d.ts
export declare function fooFromAtTypesIndex(): void;
// @Filename: /home/src/workspaces/project/node_modules/@types/dependency/lib/lol.d.ts
export declare function fooFromAtTypesLol(): void;
// @Filename: /home/src/workspaces/project/src/foo.ts
fooFrom/**/"#;
    let mut s = Session::new_for_test("autoImportProvider_exportMap6", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
