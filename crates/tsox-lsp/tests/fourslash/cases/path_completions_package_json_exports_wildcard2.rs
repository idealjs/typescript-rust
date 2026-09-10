use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_exports_wildcard2() {
    let content = r#"// @module: node18
// @Filename: /node_modules/salesforce-pageobjects/package.json
{
  "name": "salesforce-pageobjects",
  "version": "1.0.0",
  "exports": {
    "./*": {
      "types": "./dist/*.d.ts",
      "import": "./dist/*.mjs",
      "default": "./dist/*.js"
    }
  }
}
// @Filename: /node_modules/salesforce-pageobjects/dist/action/pageObjects/actionRenderer.d.ts
export const actionRenderer = 0;
// @Filename: /index.mts
import { } from "salesforce-pageobjects//**/";"#;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonExportsWildcard2", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "action/");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "pageObjects/");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
