use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_filter_text2() {
    let content = r#"// @strict: true
declare const foo1: { bar: string } | undefined;
if (true) {
    foo1[|.|]/*1*/
}
else {
    foo1?./*2*/
}
"#;
    let mut s = Session::new_for_test("completionFilterText2", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
