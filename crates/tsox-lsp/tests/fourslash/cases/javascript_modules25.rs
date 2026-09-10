use tsox_lsp::fourslash::{self, Session};


#[test]
fn javascript_modules25() {
    let content = r#"// @allowJs: true
// @Filename: mod.js
function foo() { return {a: true}; }
module.exports.a = foo;
// @Filename: app.js
import * as mod from "./mod"
mod./**/"#;
    let mut s = Session::new_for_test("javascriptModules25", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["a"], &[]);
}
