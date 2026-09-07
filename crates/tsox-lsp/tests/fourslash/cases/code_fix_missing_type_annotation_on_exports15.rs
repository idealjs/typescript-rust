use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports15() {
    let content = r#"// @stableTypeOrdering: true
// @isolatedDeclarations: true
// @declaration: true
function foo() {
    return { x: 1, y: 1 } as const;
}
export const { x, y = 0 } = foo();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
