use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_imports_wildcard6() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "name": "foo",
  "main": "dist/index.js",
  "module": "dist/index.mjs",
  "types": "dist/index.d.ts",
  "imports": {
    "#*": "./dist/*?.d.ts"
  }
}
// @Filename: /dist/index.d.ts
export const index = 0;
// @Filename: /dist/blah?.d.ts
export const blah = 0;
// @Filename: /index.mts
import { } from "/**/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsWildcard6", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
