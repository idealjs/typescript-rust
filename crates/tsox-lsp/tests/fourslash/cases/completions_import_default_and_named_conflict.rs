use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_default_and_named_conflict() {
    let content = r#"// @noLib: true
// @Filename: /someModule.ts
export const someModule = 0;
export default 1;
// @Filename: /index.ts
someMo/**/"#;
    let mut s = Session::new_for_test("completionsImport_defaultAndNamedConflict", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
