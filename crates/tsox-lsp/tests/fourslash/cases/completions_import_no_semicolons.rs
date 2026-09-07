use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_no_semicolons() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /a.ts
export function foo() {}
// @Filename: /b.ts
const x = 0
const y = 1
const z = fo/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
