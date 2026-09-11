use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports35_variable_releative() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @Filename: /code.ts
const foo = { a: 1 }
export const exported = foo;"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports35_variable_releative", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
