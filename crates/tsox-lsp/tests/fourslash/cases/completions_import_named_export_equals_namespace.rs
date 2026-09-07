use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
