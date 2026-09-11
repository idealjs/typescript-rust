use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_default_false_positive() {
    let content = r#"// @Filename: /node_modules/foo/index.ts
export default function f(): void;
// @Filename: /node_modules/bar/concat.d.ts
export const concat = 0;
// @Filename: /a.ts
export {};
conca/**/"#;
    let mut s = Session::new_for_test("completionsImport_defaultFalsePositive", content);
    // TODO: prefs := lsutil.NewDefaultUserPreferences()
    // TODO: prefs.AutoImportEntrypointDirectorySearch = core.TSTrue
    // TODO: f.Configure(t, prefs)
    fourslash::go_to_file(&mut s, "/a.ts");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
