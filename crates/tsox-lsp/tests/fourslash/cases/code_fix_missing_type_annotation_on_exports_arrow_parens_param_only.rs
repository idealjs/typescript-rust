use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports_arrow_parens_param_only() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
export const func = /*a*/x/*b*/ => 0;"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "a");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
