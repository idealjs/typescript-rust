use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_missing_type_annotation_on_exports53_nested_generic_types() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
export interface Foo<T, U = T[]> {}
export function foo(x: Map<number, Foo<string>>) {
    return x;
}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports53_nested_generic_types", content);
    // TODO: f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
