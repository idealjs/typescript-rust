use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports39_extract_arr_to_variable() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2019
// @Filename: /code.ts
let c: string[] = [];
export let o = {
    p: [
        ...c
    ]
}"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports39_extract_arr_to_variable", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
