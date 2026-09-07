use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: prefs := lsutil.NewDefaultUserPreferences()"]
#[test]
fn completions_import_default_false_positive() {
    let content = r#"// @Filename: /node_modules/foo/index.ts
export default function f(): void;
// @Filename: /node_modules/bar/concat.d.ts
export const concat = 0;
// @Filename: /a.ts
export {};
conca/**/"#;
    let mut s = Session::new(content);
    // TODO: prefs := lsutil.NewDefaultUserPreferences()
    // TODO: prefs.AutoImportEntrypointDirectorySearch = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, prefs)
    fourslash::go_to_file(&mut s, "/a.ts");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
