use tsox_lsp::fourslash::{self, Session};


#[test]
fn exhaustive_case_completions3() {
    let content = r#"// @newline: LF
// @Filename: /main.ts
enum E {
    A = 0,
    B = "B",
    C = "C",
}
declare const u: E;
switch (u) {
    case/*1*/
}
switch (u) {
    /*2*/
}
switch (u) {
    case 1:
    /*3*/
}
switch (u) {
    [|c|]/*4*/   
}
switch (u) {
    case /*5*/
}
/*6*/
switch (u) {
    /*7*/

switch (u) {
    case E./*8*/
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: exhaustiveCaseCompletion := &lsproto.CompletionItem{
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("8"), &["A", "B", "C"]);
    // TODO: }
}
