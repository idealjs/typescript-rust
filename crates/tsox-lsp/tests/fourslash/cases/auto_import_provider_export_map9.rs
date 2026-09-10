use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn auto_import_provider_export_map9() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /home/src/workspaces/project/tsconfig.json
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
  }
}
// @Filename: /home/src/workspaces/project/node_modules/dependency/package.json
{
  "type": "module",
  "name": "dependency",
  "version": "1.0.0",
  "exports": {
    "./lol": ["./lib/index.js", "./lib/lol.js"]
  }
}
// @Filename: /home/src/workspaces/project/node_modules/dependency/lib/index.d.ts
export function fooFromIndex(): void;
// @Filename: /home/src/workspaces/project/node_modules/dependency/lib/lol.d.ts
export function fooFromLol(): void;
// @Filename: /home/src/workspaces/project/src/bar.ts
import { fooFromIndex } from "dependency";
// @Filename: /home/src/workspaces/project/src/foo.ts
fooFrom/**/"#;
    let mut s = Session::new_for_test("autoImportProvider_exportMap9", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
