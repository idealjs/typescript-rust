use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_imports_bundler_no_node_condition() {
    let content = r##"// @moduleResolution: bundler
// @Filename: /package.json
{
  "name": "foo",
  "imports": {
    "#only-for-node": {
      "node": "./something.js"
    },
    "#for-everywhere": "./other.js"
  }
}
// @Filename: /something.d.ts
export const index = 0;
// @Filename: /other.d.ts
export const index = 0;
// @Filename: /index.ts
import { } from "/**/";"##;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
