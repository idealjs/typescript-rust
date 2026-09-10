use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_imports_ignore_matching_node_module2() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#internal/*": "./src/*.ts"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /src/node_modules/#internal/package.json
{}
// @Filename: /src/a.ts
import {} from "#internal//*1*/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsIgnoreMatchingNodeModule2", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
