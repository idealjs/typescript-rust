use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_exports_wildcard10() {
    let content = r#"// @module: preserve
// @moduleResolution: bundler
// @allowImportingTsExtensions: true
// @jsx: react
// @Filename: /node_modules/repo/package.json
{
  "name": "repo",
  "exports": {
    "./*": "./src/*"
  }
}
// @Filename: /node_modules/repo/src/card.tsx
export {};
// @Filename: /main.ts
import { } from "repo//**/";"#;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonExportsWildcard10", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
