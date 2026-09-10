use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // Verify that only one code fix action is returned, not thr"]
#[test]
fn code_fix_missing_type_annotation_on_exports_expando_no_duplicates() {
    let content = r#"// @declaration: true
// @isolatedDeclarations: true
// @Filename: /foo.mts
export function foo(): void {
}

foo.blah = 123;"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports_expandoNoDuplicates", content);
    // TODO: // Verify that only one code fix action is returned, not three identical ones.
    fourslash::unsupported("VerifyCodeFixAvailableExact"); // f.VerifyCodeFixAvailableExact(t, []string{
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
