use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports32_inline_short_hand() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @Filename: /code.ts
const x = 1;
export default {
  x
};"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports32_inline_short_hand", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
