use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn java_script_modules_with_backticks() {
    let content = r#"// @allowJs: true
// @Filename: a.js
exports.x = 0;
// @Filename: consumer.js
var a = require(` + "`" + `./a` + "`" + `);
a./**/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
