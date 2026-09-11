use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports22_formatting() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
/**
 * Test
 */
export function foo(){}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports22_formatting", content);
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Add return type 'void'"})
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
