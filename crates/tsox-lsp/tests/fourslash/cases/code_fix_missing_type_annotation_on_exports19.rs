use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports19() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
export const a = {
    z: Symbol()
} as const;"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports19", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
