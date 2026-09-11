use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_default_export_default_identifier() {
    let content = r#"// @module: esnext
// @Filename: /a.ts
const foo = 0;
export default foo;
// @Filename: /b.ts
f/**/;"#;
    let mut s = Session::new_for_test("completionsImport_default_exportDefaultIdentifier", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
