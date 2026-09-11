use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_assert_keyword() {
    let content = r#"// @allowJs: true
// @Filename: a.ts
 const f = {
    a: 1
};
 import * as thing from "thing" /*0*/
 export { foo } from "foo" /*1*/
 import "foo" as /*2*/
 import "foo" a/*3*/
 import * as that from "that"
 /*4*/
 import * /*5*/ as those from "those"
// @Filename: b.js
 import * as thing from "thing" /*js*/;"#;
    let mut s = Session::new_for_test("completionsAssertKeyword", content);
    fourslash::go_to_marker(&mut s, "0");
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "5");
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "js");
    // TODO: f.VerifyCompletions(t, "js", &fourslash.CompletionsExpectedList{
}
