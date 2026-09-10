use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn import_completions_package_json_imports_pattern() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#*": "./src/*"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /a.ts
import {} from "/*1*/";"##;
    let mut s = Session::new_for_test("importCompletionsPackageJsonImportsPattern", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
