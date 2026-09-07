use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
