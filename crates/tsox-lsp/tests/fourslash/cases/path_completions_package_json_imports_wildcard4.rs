use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_imports_wildcard4() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "types": "index.d.ts",
  "imports": {
    "#*": "dist/*",
    "#foo/*": "dist/*",
    "#bar/*": "dist/*",
    "#exact-match": "dist/index.d.ts"
  }
}
// @Filename: /nope.d.ts
export const nope = 0;
// @Filename: /dist/index.d.ts
export const index = 0;
// @Filename: /dist/blah.d.ts
export const blah = 0;
// @Filename: /dist/foo/onlyInFooFolder.d.ts
export const foo = 0;
// @Filename: /dist/subfolder/one.d.ts
export const one = 0;
// @Filename: /a.mts
import { } from "/**/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsWildcard4", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "#foo/");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "foo/");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
