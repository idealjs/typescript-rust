use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_specifier_exclude_regexes() {
    let content = r#"// @Filename: foo.ts
export const mySymbol = 1;
// @Filename: ignoreme.ts
export const ignoredSymbol = 2;
// @Filename: bar.ts
mySym/*1*/
ignoredSym/*2*/"#;
    let mut s = Session::new_for_test("autoImportSpecifierExcludeRegexes", content);
    // TODO: f.Configure(t, lsutil.UserPreferences{
    // TODO: // Verify that mySymbol is included, but ignoredSymbol is excluded from completions
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: // Baseline the auto-imports
    // TODO: f.BaselineAutoImportsCompletions(t, []string{"1", "2"})
}
