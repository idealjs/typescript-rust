use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_missing_type_annotation_on_exports58_genercs_doesnt_drop_trailing_unknown_2() {
    let content = r#"// @isolatedDeclarations: true
// @declaration: true
// @lib: es2015

export const s = new Set<unknown>();
"#;
    let mut s = Session::new_for_test("codeFixMissingTypeAnnotationOnExports58_genercs_doesnt_drop_trailing_unknown_2", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
