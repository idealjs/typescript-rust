use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_in_export_module_block() {
    let content = r#"const outOfScope = 0;

declare module 'mod' {
  const a: string;
  type T = number;
  export { /**/ };
}"#;
    let mut s = Session::new_for_test("completionsInExport_moduleBlock", content);
    fourslash::go_to_marker(&mut s, "");
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
