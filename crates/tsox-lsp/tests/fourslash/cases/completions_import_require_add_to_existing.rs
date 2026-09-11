use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_require_add_to_existing() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
const x = 0;
function f() {}
module.exports = { x, f };
// @Filename: /b.js
const { f } = require("./a");

x/**/"#;
    let mut s = Session::new_for_test("completionsImport_require_addToExisting", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
