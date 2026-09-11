use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_completions_package_json_exports_specifier_ends_in_ts() {
    let content = r#"// @module: node18
// @Filename: /node_modules/pkg/package.json
{
    "name": "pkg",
    "version": "1.0.0",
    "exports": {
      "./something.ts": "./a.js"
    }
 }
// @Filename: /node_modules/pkg/a.d.ts
export function foo(): void;
// @Filename: /package.json
{
    "dependencies": {
       "pkg": "*"
    }
 }
// @Filename: /index.ts
import {} from "pkg//*1*/";"#;
    let mut s = Session::new_for_test("importCompletionsPackageJsonExportsSpecifierEndsInTs", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
