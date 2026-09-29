use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_require() {
    let content = r#"// @allowJs: true
// @Filename: /a.ts
export const foo = 0;
// @Filename: /b.js
import * as s from "something";
fo/*b*/"#;
    let mut s = Session::new_for_test("completionsImport_require", content);
    fourslash::go_to_marker(&mut s, "b");
    // TODO: f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new("b"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
