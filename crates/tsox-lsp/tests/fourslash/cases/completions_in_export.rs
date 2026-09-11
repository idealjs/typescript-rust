use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_in_export() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"const a = "a";
type T = number;
export { /**/ };"#;
    let mut s = Session::new_for_test("completionsInExport", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "a, ");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "T as ");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "U, ");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "T, ");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
