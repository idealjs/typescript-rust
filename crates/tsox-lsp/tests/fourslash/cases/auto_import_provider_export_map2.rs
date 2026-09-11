use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn auto_import_provider_export_map2() {
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
{
  "compilerOptions": {
    "lib": ["es5"],
    "module": "commonjs",
    "moduleResolution": "node10"
  }
}
// @Filename: /home/src/workspaces/project/package.json
{
  "type": "module",
  "dependencies": {
    "dependency": "^1.0.0"
  }
}
// @Filename: /home/src/workspaces/project/node_modules/dependency/package.json
{
  "type": "module",
  "name": "dependency",
  "version": "1.0.0",
  "types": "./lib/index.d.ts",
  "exports": {
    ".": {
      "types": "./lib/index.d.ts"
    },
    "./lol": {
      "types": "./lib/lol.d.ts"
    }
  }
}
// @Filename: /home/src/workspaces/project/node_modules/dependency/lib/index.d.ts
export function fooFromIndex(): void;
// @Filename: /home/src/workspaces/project/node_modules/dependency/lib/lol.d.ts
export function fooFromLol(): void;
// @Filename: /home/src/workspaces/project/src/foo.ts
fooFrom/**/"#;
    let mut s = Session::new_for_test("autoImportProvider_exportMap2", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
