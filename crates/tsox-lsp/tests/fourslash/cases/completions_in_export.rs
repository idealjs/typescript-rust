use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_in_export() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"const a = "a";
type T = number;
export { /**/ };"#;
    let mut s = Session::new_for_test("completionsInExport", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "a, ");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "T as ");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "U, ");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "T, ");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
