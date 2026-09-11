use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_require_add_new() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
const x = 0;
module.exports = { x };
// @Filename: /b.js
x/**/"#;
    let mut s = Session::new_for_test("completionsImport_require_addNew", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
