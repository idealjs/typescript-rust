use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_imports_wildcard8() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "name": "foo",
  "imports": {
    "#*": "./dist/*.js"
  }
}
// @Filename: /dist/blah.js
export const blah = 0;
// @Filename: /dist/blah.d.ts
export declare const blah: 0;
// @Filename: /index.mts
import { } from "/**/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsWildcard8", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
