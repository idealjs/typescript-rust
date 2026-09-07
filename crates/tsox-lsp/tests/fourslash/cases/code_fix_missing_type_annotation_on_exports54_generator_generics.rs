use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports54_generator_generics() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2015
export function foo(x: Generator<number>) {
    return x;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
