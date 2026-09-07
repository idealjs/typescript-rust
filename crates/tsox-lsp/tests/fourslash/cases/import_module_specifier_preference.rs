use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
#[test]
fn import_module_specifier_preference_shortest() {
    let content = r#"// @Filename: /project/src/utils/helper.ts
export const helperFunc = () => {};
// @Filename: /project/src/index.ts
helper/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}

#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
#[test]
fn import_module_specifier_preference_project_relative() {
    let content = r#"// @Filename: /project/src/utils/helper.ts
export const helperFunc = () => {};
// @Filename: /project/tests/index.ts
helper/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}

#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
#[test]
fn import_module_specifier_preference_relative() {
    let content = r#"// @Filename: /project/src/utils/helper.ts
export const helperFunc = () => {};
// @Filename: /project/src/index.ts
helper/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}

#[ignore = "generator: // Regression test: when `paths` aliases are configured but "]
#[test]
fn import_module_specifier_preference_project_relative_with_paths() {
    // TODO: // Regression test: when `paths` aliases are configured but the import target
    // TODO: // lives in the same package as the importing file, `project-relative` must
    // TODO: // prefer the relative path rather than fall back to the `paths` alias.
    let content = r#"// @Filename: /project/tsconfig.json
{
  "compilerOptions": {
    "paths": {
      "@app/*": ["./src/app/*"],
      "@utils/*": ["./src/utils/*"],
    }
  }
}
// @Filename: /project/src/utils/helper.ts
export const helperFunc = () => {};
// @Filename: /project/src/app/index.ts
helper/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}

#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
#[test]
fn import_module_specifier_preference_non_relative() {
    let content = r#"// @Filename: /project/tsconfig.json
{
  "compilerOptions": {
    "paths": {
      "@app/*": ["./src/app/*"],
      "@utils/*": ["./src/utils/*"],
    }
  }
}
// @Filename: /project/src/utils/helper.ts
export const helperFunc = () => {};
// @Filename: /project/src/app/index.ts
helper/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("Configure"); // f.Configure(t, lsutil.UserPreferences{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}
