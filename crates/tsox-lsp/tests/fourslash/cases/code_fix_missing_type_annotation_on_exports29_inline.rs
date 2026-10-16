use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports29_inline() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function getString() {
    return ""
}
export const exp = {
    prop: getString()
};"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports29_inline", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
