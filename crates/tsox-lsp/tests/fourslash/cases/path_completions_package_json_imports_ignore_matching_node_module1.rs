use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
