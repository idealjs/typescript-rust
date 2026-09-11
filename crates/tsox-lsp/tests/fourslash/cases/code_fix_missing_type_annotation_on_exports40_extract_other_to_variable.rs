use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports40_extract_other_to_variable() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
// @Filename: /code.ts
let c: string[] = [];
export let o = {
    p: Math.random() ? []: [
        ...c
    ]
}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports40_extract_other_to_variable", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
