use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn java_script_modules18() {
    let content = r#"// @allowJs: true
// @Filename: myMod.js
var x = require('fs');
// @Filename: other.js
/**/;"#;
    let mut s = Session::new_for_test("javaScriptModules18", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
