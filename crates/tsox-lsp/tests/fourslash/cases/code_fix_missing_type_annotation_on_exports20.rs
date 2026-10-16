use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports20() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
export function foo () {
    return Symbol();
}"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports20", content);
    // TODO: f.VerifyCodeFixAvailable(t, []string{"Add return type 'symbol'"})
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
