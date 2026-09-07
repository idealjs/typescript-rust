use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
