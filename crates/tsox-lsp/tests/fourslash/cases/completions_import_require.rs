use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_import_require() {
    let content = r#"// @allowJs: true
// @Filename: /a.ts
export const foo = 0;
// @Filename: /b.js
import * as s from "something";
fo/*b*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new("b"), &fourslash.ApplyCodeActionFromCompletionOptions{
}
