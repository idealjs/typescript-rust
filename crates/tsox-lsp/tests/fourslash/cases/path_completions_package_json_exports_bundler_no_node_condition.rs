use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_exports_bundler_no_node_condition() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /node_modules/foo/package.json
{
  "name": "foo",
  "exports": {
    "./only-for-node": {
      "node": "./something.js"
    },
    "./for-everywhere": "./other.js"
  }
}
// @Filename: /node_modules/foo/something.d.ts
export const index = 0;
// @Filename: /node_modules/foo/other.d.ts
export const index = 0;
// @Filename: /index.ts
import { } from "foo//**/";"#;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonExportsBundlerNoNodeCondition", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
