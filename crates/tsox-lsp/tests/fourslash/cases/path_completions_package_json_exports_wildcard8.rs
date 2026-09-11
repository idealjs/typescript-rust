use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_exports_wildcard8() {
    let content = r#"// @module: node18
// @Filename: /node_modules/foo/package.json
{
  "name": "foo",
  "exports": {
    "./*": "./dist/*.js"
  }
}
// @Filename: /node_modules/foo/dist/blah.js
export const blah = 0;
// @Filename: /node_modules/foo/dist/blah.d.ts
export declare const blah: 0;
// @Filename: /index.mts
import { } from "foo//**/";"#;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonExportsWildcard8", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
