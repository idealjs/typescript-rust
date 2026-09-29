use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports13() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function foo() {
    return { x: 1, y: 1 };
}
export const { x: abcd, y: defg } = foo();"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports13", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
