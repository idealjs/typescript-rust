use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
