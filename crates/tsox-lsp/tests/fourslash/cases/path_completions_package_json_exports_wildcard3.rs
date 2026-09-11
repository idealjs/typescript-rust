use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_exports_wildcard3() {
    let content = r#"// @module: node18
// @Filename: /node_modules/foo/package.json
{
  "types": "index.d.ts",
  "exports": {
    "./component-*": {
      "types@>=4.3.5": "types/components/*.d.ts"
    }
  }
}
// @Filename: /node_modules/foo/nope.d.ts
export const nope = 0;
// @Filename: /node_modules/foo/types/components/index.d.ts
export const index = 0;
// @Filename: /node_modules/foo/types/components/blah.d.ts
export const blah = 0;
// @Filename: /node_modules/foo/types/components/subfolder/one.d.ts
export const one = 0;
// @Filename: /a.ts
import { } from "foo//**/";"#;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonExportsWildcard3", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "component-subfolder/");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
