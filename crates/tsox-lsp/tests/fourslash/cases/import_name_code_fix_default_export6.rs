use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_default_export6() {
    let content = r#"// @Filename: /a.ts
export default Math.foo;
// @Filename: /index.ts
a/**/"#;
    let mut s = Session::new_for_test("importNameCodeFixDefaultExport6", content);
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
