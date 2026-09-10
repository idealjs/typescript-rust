use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
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
    let mut s = Session::new_for_test("completionsImport_importType", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"0", "1"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("0"), &fourslash.ApplyCodeActionFromCompletionOptions{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
