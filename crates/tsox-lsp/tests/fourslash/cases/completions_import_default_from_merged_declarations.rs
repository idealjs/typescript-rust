use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_default_from_merged_declarations() {
    let content = r#"// @module: esnext
// @Filename: /a.ts
declare module "m" {
    export default class M {}
}
// @Filename: /b.ts
declare module "m" {
    export default interface M {}
}
// @Filename: /c.ts
/**/"#;
    let mut s = Session::new_for_test("completionsImport_default_fromMergedDeclarations", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
