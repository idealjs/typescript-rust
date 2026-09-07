use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_import_yield_expression() {
    let content = r#"// @Filename: /a.ts
export function a() {}
// @Filename: /b.ts
function *f() {
  yield a/**/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
