use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports6() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function foo(): number[] { return [42]; }
export const c = [...foo()];"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports6", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
