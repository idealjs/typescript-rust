use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_filter_text3() {
    let content = r#"// @strict: true
declare const foo1: { b: number; "a bc": string; };
if (true) {
    foo1[|.|]/*1*/
} 
else {
    foo1[|.a|]/*2*/
}

declare const foo2: { b: number; "a bc": string; } | undefined;
if (true) {
    foo2[|.|]/*3*/
} else if (false) {
    foo2[|.a|]/*4*/
} else {
    foo2[|?.|]/*5*/
}
"#;
    let mut s = Session::new_for_test("completionFilterText3", content);
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
}
