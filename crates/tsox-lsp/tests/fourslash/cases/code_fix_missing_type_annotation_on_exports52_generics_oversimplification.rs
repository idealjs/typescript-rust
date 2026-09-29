use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_missing_type_annotation_on_exports52_generics_oversimplification() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
export interface Foo<T, U = T[]> {}
export function foo(x: Foo<string, string[]>) {
    return x;
}"#;
    let _s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports52_generics_oversimplification", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
