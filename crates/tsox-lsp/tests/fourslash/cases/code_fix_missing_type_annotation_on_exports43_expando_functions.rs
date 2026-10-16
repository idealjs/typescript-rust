use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports43_expando_functions() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
// @Filename: /code.ts
const foo = (): void => {}
foo.a = "A";
foo.b = "C""#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports43_expando_functions", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
