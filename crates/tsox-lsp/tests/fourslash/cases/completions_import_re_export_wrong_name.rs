use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_re_export_wrong_name() {
    let content = r#"// @moduleResolution: bundler
// @Filename: /a.ts
export const x = 0;
// @Filename: /index.ts
export { x as y } from "./a";
// @Filename: /c.ts
/**/"#;
    let mut s = Session::new_for_test("completionsImport_reExport_wrongName", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
