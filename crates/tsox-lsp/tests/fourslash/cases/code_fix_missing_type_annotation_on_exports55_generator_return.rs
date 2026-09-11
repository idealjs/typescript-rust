use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports55_generator_return() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2015
export function *foo() {
    yield 5;
}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports55_generator_return", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
