use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_imports_wildcard11() {
    let content = r##"// @module: preserve
// @moduleResolution: bundler
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
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsWildcard11", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
