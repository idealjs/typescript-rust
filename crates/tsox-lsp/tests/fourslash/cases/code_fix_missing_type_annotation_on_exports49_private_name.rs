use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports49_private_name() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @moduleResolution: bundler
// @target: es2018
// @jsx: react-jsx
export function two() {
    const y = "";
    return {} as typeof y;
}

export function three() {
    type Z = string;
    return {} as Z;
}"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports49_private_name", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
