use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_imports_wildcard2() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "name": "salesforce-pageobjects",
  "version": "1.0.0",
  "imports": {
    "#*": {
      "types": "./dist/*.d.ts",
      "import": "./dist/*.mjs",
      "default": "./dist/*.js"
    }
  }
}
// @Filename: /dist/action/pageObjects/actionRenderer.d.ts
export const actionRenderer = 0;
// @Filename: /index.mts
import { } from "/**/";"##;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "#action/");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "pageObjects/");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
