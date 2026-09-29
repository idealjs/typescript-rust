use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function foo() { return 42; }
export const g = foo();"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
