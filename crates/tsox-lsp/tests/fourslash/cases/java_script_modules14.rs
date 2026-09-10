use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn java_script_modules14() {
    let content = r#"// @allowJs: true
// @Filename: myMod.js
if (true) {
    exports.b = true;
} else {
    exports.n = 3;
}
function fn() {
    exports.s = 'foo';
}
var invisible = true;
// @Filename: isGlobal.js
var y = 10;
// @Filename: consumer.js
var x = require('myMod');
/**/;"#;
    let mut s = Session::new_for_test("javaScriptModules14", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
