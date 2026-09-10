use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn java_script_modules13() {
    let content = r#"// @allowJs: true
// @Filename: myMod.js
if (true) {
    module.exports = { a: 10 };
}
var invisible = true;
// @Filename: isGlobal.js
var y = 10;
// @Filename: consumer.js
var x = require('./myMod');
/**/;"#;
    let mut s = Session::new_for_test("javaScriptModules13", content);
    fourslash::go_to_file(&mut s, "consumer.js");
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "x.");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "a.");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
