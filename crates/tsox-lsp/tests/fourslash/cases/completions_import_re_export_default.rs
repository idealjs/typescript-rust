use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_re_export_default() {
    let content = r#"// @lib: es5
// @module: esnext
// @Filename: /a/b/impl.ts
export default function foo() {}
// @Filename: /a/index.ts
export { default as foo } from "./b/impl";
// @Filename: /use.ts
fo/**/"#;
    let mut s = Session::new_for_test("completionsImport_reExportDefault", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
