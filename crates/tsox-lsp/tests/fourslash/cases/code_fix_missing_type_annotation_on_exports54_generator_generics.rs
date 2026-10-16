use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports54_generator_generics() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2015
export function foo(x: Generator<number>) {
    return x;
}"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports54_generator_generics", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
