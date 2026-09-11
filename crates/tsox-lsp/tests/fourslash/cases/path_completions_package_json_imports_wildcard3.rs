use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_imports_wildcard3() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "types": "index.d.ts",
  "imports": {
    "#component-*": {
      "types@>=4.3.5": "types/components/*.d.ts"
    }
  }
}
// @Filename: /nope.d.ts
export const nope = 0;
// @Filename: /types/components/index.d.ts
export const index = 0;
// @Filename: /types/components/blah.d.ts
export const blah = 0;
// @Filename: /types/components/subfolder/one.d.ts
export const one = 0;
// @Filename: /a.ts
import { } from "/**/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsWildcard3", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "#component-subfolder/");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
