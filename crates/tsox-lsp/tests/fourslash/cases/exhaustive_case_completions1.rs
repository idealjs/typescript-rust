use tsox_lsp::fourslash::{self, Session};


#[test]
fn exhaustive_case_completions1() {
    let content = r#"// @newline: LF
enum E {
    A = 0,
    B = "B",
    C = "C",
}
// Mixed union
declare const u: E.A | E.B | 1;
switch (u) {
    case/*1*/
}
// Union enum
declare const e: E;
switch (e) {
    case/*2*/
}
enum F {
    D = 1 << 0,
    E = 1 << 1,
    F = 1 << 2,
}

declare const f: F;
switch (f) {
    case/*3*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
