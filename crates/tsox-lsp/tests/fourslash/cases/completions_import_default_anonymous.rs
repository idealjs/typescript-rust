use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_default_anonymous() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @module: esnext
// @noLib: true
// @Filename: /src/foo-bar.ts
export default 0;
// @Filename: /src/b.ts
def/*0*/
fooB/*1*/"#;
    let mut s = Session::new_for_test("completionsImport_default_anonymous", content);
    fourslash::go_to_marker(&mut s, "0");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("1"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
