use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_exports_wildcard11() {
    let content = r#"// @module: preserve
// @moduleResolution: bundler
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
    let mut s = Session::new_for_test("pathCompletionsPackageJsonExportsWildcard11", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
