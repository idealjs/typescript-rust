use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports58_genercs_doesnt_drop_trailing_unknown_2() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2015

export const s = new Set<unknown>();
"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports58_genercs_doesnt_drop_trailing_unknown_2", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
