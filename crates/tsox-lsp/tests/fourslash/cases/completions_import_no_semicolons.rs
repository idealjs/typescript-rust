use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_import_no_semicolons() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /a.ts
export function foo() {}
// @Filename: /b.ts
const x = 0
const y = 1
const z = fo/**/"#;
    let mut s = Session::new_for_test("completionsImport_noSemicolons", content);
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
