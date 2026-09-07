use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
#[test]
fn import_module_specifier_ending_auto() {
    let content = r#"// @Filename: /project/helper/index.ts
export const helperFunc = () => {};
// @Filename: /project/index.ts
helper/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}

#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
#[test]
fn import_module_specifier_ending_minimal() {
    let content = r#"// @Filename: /project/helper/index.ts
export const helperFunc = () => {};
// @Filename: /project/index.ts
helper/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}

#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
#[test]
fn import_module_specifier_ending_index() {
    let content = r#"// @Filename: /project/helper/index.ts
export const helperFunc = () => {};
// @Filename: /project/index.ts
helper/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}

#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
#[test]
fn import_module_specifier_ending_js() {
    let content = r#"// @Filename: /project/helper/index.ts
export const helperFunc = () => {};
// @Filename: /project/index.ts
helper/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}
