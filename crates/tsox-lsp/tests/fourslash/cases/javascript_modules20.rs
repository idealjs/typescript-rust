use tsox_lsp::fourslash::{self, Session};


#[test]
fn javascript_modules20() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @Filename: mod.js
function foo() { return {a: true}; }
module.exports = foo();
// @Filename: app.js
import * as mod from "./mod"
mod./**/"#;
    let mut s = Session::new_for_test("javascriptModules20", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
