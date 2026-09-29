use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_export() {
    let content = r#"// @typeRoots: fourslash/my_typings
// @Filename: fourslash/test.ts
export * from "./some/*0*/
export * from "./sub/some/*1*/";
export * from "[|some-/*2*/|]";
export * from "..//*3*/";
export {} from ".//*4*/";
// @Filename: fourslash/someFile1.ts
/*someFile1*/
// @Filename: fourslash/sub/someFile2.ts
/*someFile2*/
// @Filename: fourslash/my_typings/some-module/index.d.ts
export var x = 9;"#;
    let mut s = Session::new_for_test("completionForStringLiteralExport", content);
    // TODO: f.VerifyCompletions(t, []string{"0", "4"}, &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["someFile2"]);
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["fourslash"]);
}
