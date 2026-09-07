use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn javascript_modules25() {
    let content = r#"// @allowJs: true
// @Filename: mod.js
function foo() { return {a: true}; }
module.exports.a = foo;
// @Filename: app.js
import * as mod from "./mod"
mod./**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
