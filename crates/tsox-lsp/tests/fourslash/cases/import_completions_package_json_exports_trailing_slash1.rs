use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn import_completions_package_json_exports_trailing_slash1() {
    let content = r#"// @module: node18
// @moduleResolution: nodenext
// @Filename: /node_modules/pkg/package.json
{
    "name": "pkg",
    "version": "1.0.0",
    "exports": {
      "./test/": "./"
    }
 }
// @Filename: /node_modules/pkg/foo.d.ts
export function foo(): void;
// @Filename: /package.json
{
    "dependencies": {
       "pkg": "*"
    }
 }
// @Filename: /index.ts
import {} from "pkg//*1*/";
import {} from "pkg/test//*2*/";"#;
    let mut s = Session::new_for_test("importCompletionsPackageJsonExportsTrailingSlash1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
}
