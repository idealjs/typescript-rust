use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("importCompletionsPackageJsonExportsTrailingSlash1", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
}
