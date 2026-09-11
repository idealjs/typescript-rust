use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports43_expando_functions_3() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
// @Filename: /code.ts
function foo(): void {}
foo.x = 1;
foo.y = 1;"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports43_expando_functions_3", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
