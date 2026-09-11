use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_exports_wildcard12() {
    let content = r#"// @module: node18
// @Filename: /node_modules/foo/package.json
 {
   "name": "foo",
   "exports": {
     "./bar/_*/suffix": "./dist/*.js"
   }
 }
// @Filename: /node_modules/foo/dist/b.d.ts
export const x = 0;
// @Filename: /node_modules/foo/dist/dir/x.d.ts
/export const x = 0;
// @Filename: /a.mts
import {} from "foo/bar//*0*/";
import {} from "foo/bar/dir//*1*/"; // invalid
import {} from "foo/bar/[|_|]/*2*/";
import {} from "foo/bar/_dir//*3*/";"#;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonExportsWildcard12", content);
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_empty_at(&mut s, Some("1"));
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
