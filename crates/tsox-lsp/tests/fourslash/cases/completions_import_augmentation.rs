use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_augmentation() {
    let content = r#"// @Filename: /a.ts
export const foo = 0;
// @Filename: /bar.ts
export {};
declare module "./a" {
    export const bar = 0;
}
// @Filename: /user.ts
/**/"#;
    let mut s = Session::new_for_test("completionsImport_augmentation", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
