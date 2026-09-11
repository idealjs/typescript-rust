use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_default_did_not_exist_before() {
    let content = r#"// @module: esnext
// @Filename: /a.ts
export default function foo() {}
// @Filename: /b.ts
f/**/;"#;
    let mut s = Session::new_for_test("completionsImport_default_didNotExistBefore", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
