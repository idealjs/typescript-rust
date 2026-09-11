use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_multiple_with_same_name() {
    let content = r#"// @module: esnext
// @noLib: true
// @Filename: /global.d.ts
declare var foo: number;
// @Filename: /a.ts
export const foo = 0;
// @Filename: /b.ts
export const foo = 1;
// @Filename: /c.ts
fo/**/"#;
    let mut s = Session::new_for_test("completionsImport_multipleWithSameName", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
