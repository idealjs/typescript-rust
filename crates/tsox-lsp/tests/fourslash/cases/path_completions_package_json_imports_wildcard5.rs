use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_imports_wildcard5() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "name": "foo",
  "main": "dist/index.js",
  "module": "dist/index.mjs",
  "types": "dist/index.d.ts",
  "imports": {
    "#*": {
      "import": {
        "types": "./dist/types/*.d.mts",
        "default": "./dist/esm/*.mjs"
      },
      "default": {
        "types": "./dist/types/*.d.ts",
        "default": "./dist/cjs/*.js"
      }
    },
    "#only-in-cjs": {
      "require": {
        "types": "./dist/types/only-in-cjs/index.d.ts",
        "default": "./dist/cjs/only-in-cjs/index.js"
      }
    }
  }
}
// @Filename: /dist/types/index.d.mts
export const index = 0;
// @Filename: /dist/types/index.d.ts
export const index = 0;
// @Filename: /dist/types/blah.d.mts
export const blah = 0;
// @Filename: /dist/types/blah.d.ts
export const blah = 0;
// @Filename: /dist/types/only-in-cjs/index.d.ts
export const onlyInCjs = 0;
// @Filename: /index.mts
import { } from "/**/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsWildcard5", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
