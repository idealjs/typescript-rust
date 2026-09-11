use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_exports_wildcard6() {
    let content = r#"// @module: node18
// @Filename: /node_modules/foo/package.json
{
  "name": "foo",
  "main": "dist/index.js",
  "module": "dist/index.mjs",
  "types": "dist/index.d.ts",
  "exports": {
    "./*": "./dist/*?.d.ts"
  }
}
// @Filename: /node_modules/foo/dist/index.d.ts
export const index = 0;
// @Filename: /node_modules/foo/dist/blah?.d.ts
export const blah = 0;
// @Filename: /index.mts
import { } from "foo//**/";"#;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonExportsWildcard6", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
