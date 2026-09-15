use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports_expando_no_duplicates() {
    let content = r#"// @declaration: true
// @isolatedDeclarations: true
// @Filename: /foo.mts
export function foo(): void {
}

foo.blah = 123;"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports_expandoNoDuplicates", content);
    // TODO: // Verify that only one code fix action is returned, not three identical ones.
    // TODO: f.VerifyCodeFixAvailableExact(t, []string{
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
