use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn import_name_code_fix_default_export6() {
    let content = r#"// @Filename: /a.ts
export default Math.foo;
// @Filename: /index.ts
a/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
