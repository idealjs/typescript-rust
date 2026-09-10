use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn package_json_imports_failed_lookups() {
    let content = r##"// @Filename: /a/b/c/d/e/tsconfig.json
{ "compilerOptions": { "lib": ["es5"], "module": "nodenext" } }
// @Filename: /a/b/c/d/e/package.json
{
  "name": "app",
  "imports": {
    "#utils": "lodash"
  }
}
// @Filename: /a/b/node_modules/lodash/index.d.ts
export function add(a: number, b: number): number;
// @Filename: /a/b/c/d/e/index.ts
import { add } from "#utils";"##;
    let mut s = Session::new_for_test("packageJsonImportsFailedLookups", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_file(&mut s, "/a/b/c/d/e/index.ts");
}
