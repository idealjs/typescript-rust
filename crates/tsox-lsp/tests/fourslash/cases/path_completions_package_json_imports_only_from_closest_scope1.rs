use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_package_json_imports_only_from_closest_scope1() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#thing": "./src/something.ts"
  }
}
// @Filename: /src/package.json
{}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /src/a.ts
import {} from "/*1*/";
// @Filename: /a.ts
import {} from "/*2*/";"##;
    let mut s = Session::new_for_test("pathCompletionsPackageJsonImportsOnlyFromClosestScope1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
}
