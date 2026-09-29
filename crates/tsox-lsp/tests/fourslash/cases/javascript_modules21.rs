use tsox_lsp::fourslash::{self, Session};


#[test]
fn javascript_modules21() {
    let content = r#"// @allowJs: true
// @module: system
// @Filename: mod.js
function foo() { return {a: true}; }
module.exports = foo();
// @Filename: app.js
import mod from "./mod"
mod./**/"#;
    let mut s = Session::new_for_test("javascriptModules21", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
