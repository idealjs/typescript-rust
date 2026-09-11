use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_completions_package_json_imports_pattern_caps_in_path2() {
    let content = r##"// @module: node18
// @Filename: /Dev/package.json
{
  "imports": {
    "#thing/*": "./src/*.js"
  }
}
// @Filename: /Dev/src/something.ts
export function something(name: string): any;
// @Filename: /Dev/a.ts
import {} from "#thing//*2*/";"##;
    let mut s = Session::new_for_test("importCompletionsPackageJsonImportsPattern_capsInPath2", content);
    // TODO: f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
}
