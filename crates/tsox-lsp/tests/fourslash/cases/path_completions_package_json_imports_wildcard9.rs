use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_imports_wildcard9() {
    let content = r##"// @module: node18
// @allowJs: true
// @Filename: /package.json
{
  "name": "foo",
  "imports": {
    "#*": "./dist/*.js"
  }
}
// @Filename: /dist/blah.js
export const blah = 0;
// @Filename: /index.mts
import { } from "/**/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsWildcard9", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
