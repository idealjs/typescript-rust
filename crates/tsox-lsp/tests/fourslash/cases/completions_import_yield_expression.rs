use tsox_lsp::fourslash::Session;


#[test]
fn completions_import_yield_expression() {
    let content = r#"// @Filename: /a.ts
export function a() {}
// @Filename: /b.ts
function *f() {
  yield a/**/
}"#;
    let _s = Session::new_for_test("completionsImportYieldExpression", content);
    // TODO: f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
