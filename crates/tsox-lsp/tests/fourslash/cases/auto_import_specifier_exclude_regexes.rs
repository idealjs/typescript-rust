use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // Baseline the auto-imports"]
#[test]
fn auto_import_specifier_exclude_regexes() {
    let content = r#"// @Filename: foo.ts
export const mySymbol = 1;
// @Filename: ignoreme.ts
export const ignoredSymbol = 2;
// @Filename: bar.ts
mySym/*1*/
ignoredSym/*2*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{
    // TODO: // Verify that mySymbol is included, but ignoredSymbol is excluded from completions
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: // Baseline the auto-imports
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{"1", "2"})
}
