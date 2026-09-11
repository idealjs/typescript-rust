use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_export_as_default() {
    let content = r#"// @Filename: /foo.ts
const foo = 'foo'
export { foo as default }
// @Filename: /index.ts
 foo/**/"#;
    let mut s = Session::new_for_test("importNameCodeFixExportAsDefault", content);
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
