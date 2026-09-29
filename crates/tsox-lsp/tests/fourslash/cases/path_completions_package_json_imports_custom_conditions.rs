use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_imports_custom_conditions() {
    let content = r##"// @module: node18
// @customConditions: custom-condition
// @Filename: /package.json
{
  "name": "foo",
  "imports": {
    "#only-with-custom-conditions": {
      "custom-condition": "./something.js"
    }
  }
}
// @Filename: /something.d.ts
export const index = 0;
// @Filename: /index.ts
import { } from "/**/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsCustomConditions", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
