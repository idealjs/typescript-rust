use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports27_non_exported_bidings() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
let p = { x: 1, y: 2}
const a = 1, b = 10, { x, y } = p, c = 1;
export { x, y }
export const d = a + b + c;"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports27_non_exported_bidings", content);
    // TODO: f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
