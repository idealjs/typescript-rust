use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports15() {
    let content = r#"// @stableTypeOrdering: true
// @isolatedDeclarations: true
// @declaration: true
function foo() {
    return { x: 1, y: 1 } as const;
}
export const { x, y = 0 } = foo();"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports15", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
