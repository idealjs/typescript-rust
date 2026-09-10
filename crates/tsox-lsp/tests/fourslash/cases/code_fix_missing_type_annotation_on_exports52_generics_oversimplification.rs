use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports52_generics_oversimplification() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
export interface Foo<T, U = T[]> {}
export function foo(x: Foo<string, string[]>) {
    return x;
}"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports52_generics_oversimplification", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
