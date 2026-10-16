use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports32_inline_short_hand() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @Filename: /code.ts
const x = 1;
export default {
  x
};"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports32_inline_short_hand", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
