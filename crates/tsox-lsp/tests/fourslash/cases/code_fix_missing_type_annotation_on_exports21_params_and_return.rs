use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports21_params_and_return() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
/**
 * Test
 */
export function foo(): number { return 0; }
/**
* Docs
*/
export const bar = (a = foo()) =>
   a;
// Trivia"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports21_params_and_return", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
