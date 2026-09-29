use tsox_lsp::fourslash::Session;


#[test]
fn import_completions_package_json_imports_pattern_caps_in_path1() {
    let content = r##"// @module: node18
// @Filename: /Dev/package.json
{
  "imports": {
    "#thing": "./src/something.js"
  }
}
// @Filename: /Dev/src/something.ts
export function something(name: string): any;
// @Filename: /Dev/a.ts
import {} from "/*1*/";"##;
    let _s = Session::new_for_test("importCompletionsPackageJsonImportsPattern_capsInPath1", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
