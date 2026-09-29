use tsox_lsp::fourslash::{self, Session};


#[test]
fn triple_slash_ref_path_completion_hidden_file() {
    let content = r#"// @Filename: f.ts
/*f*/
// @Filename: .hidden.ts
/*hidden*/
// @Filename: test.ts
/// <reference path="/*0*/
/// <reference path="[|./*1*/|]
/// <reference path=".//*2*/
/// <reference path=".\/*3*/"#;
    let mut s = Session::new_for_test("tripleSlashRefPathCompletionHiddenFile", content);
    // TODO: f.VerifyCompletions(t, []string{"0", "2", "3"}, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
