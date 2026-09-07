use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports53_nested_generic_types() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
export interface Foo<T, U = T[]> {}
export function foo(x: Map<number, Foo<string>>) {
    return x;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
