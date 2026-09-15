use tsox_lsp::fourslash::Session;


#[test]
fn import_completions_package_json_imports_pattern_ts_ts() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#*.ts": "./src/*.ts"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /a.ts
import {} from "/*1*/";"##;
    let _s = Session::new_for_test("importCompletionsPackageJsonImportsPattern_ts_ts", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
