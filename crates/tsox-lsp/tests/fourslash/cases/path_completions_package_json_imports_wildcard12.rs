use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_package_json_imports_wildcard12() {
    let content = r##"// @module: node18
// @Filename: /package.json
 {
   "name": "repo",
   "imports": {
     "#foo/_*/suffix": "./src/*.ts"
   }
 }
// @Filename: /src/b.ts
export const x = 0;
// @Filename: /src/dir/x.ts
/export const x = 0;
// @Filename: /src/a.ts
import {} from "#foo//*0*/";
import {} from "#foo/dir//*1*/"; // invalid
import {} from "#foo/[|_|]/*2*/";
import {} from "#foo/_dir//*3*/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsWildcard12", content);
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_empty_at(&mut s, Some("1"));
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
