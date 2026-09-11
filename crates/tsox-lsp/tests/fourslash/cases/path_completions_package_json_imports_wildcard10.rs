use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_imports_wildcard10() {
    let content = r##"// @module: preserve
// @moduleResolution: bundler
// @allowImportingTsExtensions: true
// @jsx: react
// @Filename: /package.json
{
  "name": "repo",
  "imports": {
    "#*": "./src/*"
  }
}
// @Filename: /src/card.tsx
export {};
// @Filename: /main.ts
import { } from "/**/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsWildcard10", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
