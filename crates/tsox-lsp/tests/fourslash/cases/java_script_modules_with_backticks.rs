use tsox_lsp::fourslash::{self, Session};


#[test]
fn java_script_modules_with_backticks() {
    let content = r#"// @allowJs: true
// @Filename: a.js
exports.x = 0;
// @Filename: consumer.js
var a = require(` + "`" + `./a` + "`" + `);
a./**/;"#;
    let mut s = Session::new_for_test("javaScriptModulesWithBackticks", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["x"], &[]);
}
