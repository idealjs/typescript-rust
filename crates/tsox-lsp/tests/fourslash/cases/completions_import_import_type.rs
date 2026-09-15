use tsox_lsp::fourslash::Session;


#[test]
fn completions_import_import_type() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
export const x = 0;
export class C {}
/** @typedef {number} T */
// @Filename: /b.js
export const m = 0;
/** @type {/*0*/} */
/** @type {/*1*/} */"#;
    let _s = Session::new_for_test("completionsImport_importType", content);
    // TODO: f.VerifyCompletions(t, []string{"0", "1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("0"), &fourslash.ApplyCodeActionFromCompletionOptions{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
