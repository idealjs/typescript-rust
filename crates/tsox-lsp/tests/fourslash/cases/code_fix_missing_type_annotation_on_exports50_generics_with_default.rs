use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports50_generics_with_default() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2015
let x: Iterator<number>;
export const y = x;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
