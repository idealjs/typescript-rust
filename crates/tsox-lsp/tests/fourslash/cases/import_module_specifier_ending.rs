use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_module_specifier_ending_auto() {
    let content = r#"// @Filename: /project/helper/index.ts
export const helperFunc = () => {};
// @Filename: /project/index.ts
helper/**/"#;
    let mut s = Session::new_for_test("importModuleSpecifierEndingAuto", content);
    // TODO: f.Configure(t, lsutil.UserPreferences{
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}

#[test]
fn import_module_specifier_ending_minimal() {
    let content = r#"// @Filename: /project/helper/index.ts
export const helperFunc = () => {};
// @Filename: /project/index.ts
helper/**/"#;
    let mut s = Session::new_for_test("importModuleSpecifierEndingMinimal", content);
    // TODO: f.Configure(t, lsutil.UserPreferences{
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}

#[test]
fn import_module_specifier_ending_index() {
    let content = r#"// @Filename: /project/helper/index.ts
export const helperFunc = () => {};
// @Filename: /project/index.ts
helper/**/"#;
    let mut s = Session::new_for_test("importModuleSpecifierEndingIndex", content);
    // TODO: f.Configure(t, lsutil.UserPreferences{
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}

#[test]
fn import_module_specifier_ending_js() {
    let content = r#"// @Filename: /project/helper/index.ts
export const helperFunc = () => {};
// @Filename: /project/index.ts
helper/**/"#;
    let mut s = Session::new_for_test("importModuleSpecifierEndingJs", content);
    // TODO: f.Configure(t, lsutil.UserPreferences{
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}
