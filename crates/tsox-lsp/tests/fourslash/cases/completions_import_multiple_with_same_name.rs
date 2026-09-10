use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
