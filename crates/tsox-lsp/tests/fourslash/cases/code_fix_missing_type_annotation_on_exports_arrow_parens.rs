use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn code_fix_missing_type_annotation_on_exports_arrow_parens() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
export const func = x => x.substring("foo");"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
