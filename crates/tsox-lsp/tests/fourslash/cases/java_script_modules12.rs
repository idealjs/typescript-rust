use tsox_lsp::fourslash::{self, Session};


#[test]
fn java_script_modules12() {
    let content = r#"// @allowJs: true
// @Filename: mod1.js
var x = require('fs');
/*1*/
// @Filename: mod2.js
var y;
if(true) {
    y = require('fs');
}
/*2*/
// @Filename: glob1.js
var a = require;
/*3*/
// @Filename: glob2.js
var b = '';
/*4*/
// @Filename: consumer.js
/*5*/"#;
    let mut s = Session::new_for_test("javaScriptModules12", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"5"}, &fourslash.CompletionsExpectedList{
}
