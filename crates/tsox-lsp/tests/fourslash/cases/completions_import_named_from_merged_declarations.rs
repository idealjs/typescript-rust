use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_named_from_merged_declarations() {
    let content = r#"// @module: esnext
// @Filename: /a.ts
declare module "m" {
    export class M {}
}
// @Filename: /b.ts
declare module "m" {
    export interface M {}
}
// @Filename: /c.ts
/**/"#;
    let mut s = Session::new_for_test("completionsImport_named_fromMergedDeclarations", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
