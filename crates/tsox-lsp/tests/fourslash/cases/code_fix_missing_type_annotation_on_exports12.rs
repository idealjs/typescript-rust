use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports12() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function foo() {
    return { x: 1, y: 1 };
}
export const { x, y } = foo();"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports12", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
