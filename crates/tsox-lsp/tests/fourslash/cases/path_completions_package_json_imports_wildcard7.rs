use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_imports_wildcard7() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "name": "foo",
  "imports": {
    "#*": "./dist/*.js"
  }
}
// @Filename: /dist/blah.d.ts
export const blah = 0;
// @Filename: /index.mts
import { } from "/**/";"##;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
