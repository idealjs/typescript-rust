use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn import_completions_package_json_imports_pattern_root_wildcard() {
    let content = r##"// @module: nodenext
// @Filename: /package.json
{
  "imports": {
    "#/*": "./src/*"
  }
}
// @Filename: /src/something.ts
export function something(name: string): any;
// @Filename: /src/features/bar.ts
export function bar(): any;
// @Filename: /a.ts
import {} from "#//*1*/";"##;
    let _s = Session::new_for_test("importCompletionsPackageJsonImportsPatternRootWildcard", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
}
