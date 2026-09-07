use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports14() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
function foo() {
    return { x: 1, y: 1};
}
export const { x, y = 0} = foo(), z= 42;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
