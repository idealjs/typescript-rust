use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports17_unique_symbol() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
export const a = Symbol();"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports17_unique_symbol", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
