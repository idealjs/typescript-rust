use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_named_export_equals_namespace() {
    let content = r#"// @module: esnext
// @Filename: /a.d.ts
declare namespace N {
    export const foo = 0;
}
export = N;
// @Filename: /b.ts
f/**/;"#;
    let mut s = Session::new_for_test("completionsImport_named_exportEqualsNamespace", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
