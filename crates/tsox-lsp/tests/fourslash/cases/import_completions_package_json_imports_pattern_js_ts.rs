use tsox_lsp::fourslash::Session;


#[test]
fn import_completions_package_json_imports_pattern_js_ts() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#*.js": "./src/*.ts"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /a.ts
import {} from "/*1*/";"##;
    let _s = Session::new_for_test("importCompletionsPackageJsonImportsPattern_js_ts", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
