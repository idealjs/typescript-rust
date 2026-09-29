use tsox_lsp::fourslash::Session;


#[test]
fn import_completions_package_json_imports_pattern_ts_js() {
    let content = r##"// @module: node18
// @Filename: /package.json
{
  "imports": {
    "#*.ts": "./src/*.js"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /a.ts
import {} from "/*1*/";"##;
    let _s = Session::new_for_test("importCompletionsPackageJsonImportsPattern_ts_js", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
