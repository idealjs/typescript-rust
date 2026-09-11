use tsox_lsp::fourslash::{self, Session};


#[test]
fn java_script_modules19() {
    let content = r#"// @allowJs: true
// @Filename: myMod.js
var x = { a: 10 };
module.exports = x;
// @Filename: isGlobal.js
var y = 10;
// @Filename: consumer.js
var x = require('./myMod');
/**/;"#;
    let mut s = Session::new_for_test("javaScriptModules19", content);
    fourslash::go_to_file(&mut s, "consumer.js");
    fourslash::go_to_marker(&mut s, "");
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "x.");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "a.");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
