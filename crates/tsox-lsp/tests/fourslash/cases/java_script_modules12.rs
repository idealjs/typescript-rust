use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"5"}, &fourslash.CompletionsExpectedList{
}
