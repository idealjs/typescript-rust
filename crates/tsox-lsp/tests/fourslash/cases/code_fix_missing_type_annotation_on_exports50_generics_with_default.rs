use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports50_generics_with_default() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2015
let x: Iterator<number>;
export const y = x;"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports50_generics_with_default", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
