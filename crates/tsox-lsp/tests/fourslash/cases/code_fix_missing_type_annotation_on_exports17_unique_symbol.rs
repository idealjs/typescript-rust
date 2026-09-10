use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports17_unique_symbol() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
export const a = Symbol();"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports17_unique_symbol", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
