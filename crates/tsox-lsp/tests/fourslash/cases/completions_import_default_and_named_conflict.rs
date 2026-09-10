use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_import_default_and_named_conflict() {
    let content = r#"// @noLib: true
// @Filename: /someModule.ts
export const someModule = 0;
export default 1;
// @Filename: /index.ts
someMo/**/"#;
    let mut s = Session::new_for_test("completionsImport_defaultAndNamedConflict", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
