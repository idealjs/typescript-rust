use tsox_lsp::fourslash::Session;


#[test]
fn path_completions_package_json_imports_ignore_matching_node_module1() {
    let content = r##"// @module: node18
// @Filename: /src/node_modules/#internal/package.json
{
  "imports": {
    "#thing": "./dist/something.js"
  }
}
// @Filename: /src/node_modules/#internal/dist/something.d.ts
export function something(name: string): any;
// @Filename: /src/a.ts
import {} from "#internal//*1*/";"##;
    let _s = Session::new_for_test("pathCompletionsPackageJsonImportsIgnoreMatchingNodeModule1", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
