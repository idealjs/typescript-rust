use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_exports_custom_conditions() {
    let content = r#"// @module: node18
// @customConditions: custom-condition
// @Filename: /node_modules/foo/package.json
{
  "name": "foo",
  "exports": {
    "./only-with-custom-conditions": {
      "custom-condition": "./something.js"
    }
  }
}
// @Filename: /node_modules/foo/something.d.ts
export const index = 0;
// @Filename: /index.ts
import { } from "foo//**/";"#;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonExportsCustomConditions", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
