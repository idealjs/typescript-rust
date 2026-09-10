use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
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
    fourslash::unsupported("VerifyCodeFixAvailable"); // f.VerifyCodeFixAvailable(t, []string{"Add return type 'void'"})
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
